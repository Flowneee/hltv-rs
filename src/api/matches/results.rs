use std::collections::HashMap;

use scraper::{Html, Selector};

use crate::{
    api::{ElementRef, ElementRefExt},
    Error, NoneErrorExt, Result,
};

/// Short match result.
#[derive(Debug, PartialEq)]
pub struct MatchResult {
    pub team1: String,
    pub team2: String,
    pub result: (u8, u8),
    pub link: String,
    pub event: String,
    pub map: String,
    pub stars: usize,
}

impl MatchResult {
    // TODO: doc
    pub fn from_element_ref(element: ElementRef) -> Result<Self> {
        let link = element
            .select_one("a")?
            .hltv_parse_err("No 'a' element with link to match result")?
            .value()
            .attr("href")
            .hltv_parse_err("No href to match result")?
            .into();

        let team1 = element
            .select_one("div.team1>div.team")?
            .hltv_parse_err("No team1 for match result")?
            .text2();
        let team2 = element
            .select_one("div.team2>div.team")?
            .hltv_parse_err("No team2 for match result")?
            .text2();

        let score_selector = Selector::parse("td.result-score>span")?;
        let mut score_elements = element.select(&score_selector);
        let team1_score = score_elements
            .next()
            .hltv_parse_err("No score for team1")?
            .text2()
            .parse::<u8>()
            .map_err(|_| Error::HltvParse("Score is not integer".into()))?;
        let team2_score = score_elements
            .next()
            .hltv_parse_err("No score for team2")?
            .text2()
            .parse::<u8>()
            .map_err(|_| Error::HltvParse("Score is not integer".into()))?;

        let event = element
            .select_one("span.event-name")?
            .hltv_parse_err("Failed to find event name for match result")?
            .text2();

        let map = element
            .select_one("div.map-text")?
            .hltv_parse_err("Failed to find map for match result")?
            .text2();

        let stars = element.select(&Selector::parse("i.star")?).count();

        Ok(Self {
            team1,
            team2,
            result: (team1_score, team2_score),
            link,
            event,
            map,
            stars,
        })
    }
}

/// Short batch results for multiple days.
pub struct MatchesResults {
    pub results: HashMap<String, Vec<MatchResult>>,
}

impl MatchesResults {
    // TODO: doc
    pub fn from_html(document: &Html) -> Result<Self> {
        document
            .select(&Selector::parse(
                "div.results-holder>div.results-all>div.results-sublist",
            )?)
            .map(|element| {
                let day = element
                    .select_one(".standard-headline")? // Seems like it changes from div to span in JS
                    .hltv_parse_err("Failed to find day headline")?
                    .text2();

                let results = element
                    .select(&Selector::parse("div.result-con")?)
                    .map(|x| MatchResult::from_element_ref(x))
                    .collect::<Result<Vec<_>>>()?;
                Ok((day, results))
            })
            .collect::<Result<HashMap<_, _>>>()
            .map(|results| MatchesResults { results })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use scraper::{Html, Selector};

    fn get_elem<'a>(html: &'a Html, css_selector: &'_ str) -> ElementRef<'a> {
        html.select(&Selector::parse(css_selector).unwrap())
            .next()
            .unwrap()
    }

    #[test]
    fn parse_match_result() {
        let html = Html::parse_fragment(
            r#"
<div class="result-con " data-zonedgrouping-entry-unix="1604796087000"><a href="/matches/2345169/stmn-vs-loto-fireleague-latin-power-blast-premier-qualifier" class="a-reset">
  <div class="result">
    <table>
      <tbody><tr>
        <td class="team-cell">
          <div class="line-align team1">
            <div class="team team-won">STMN</div>
ttps://img-cdn.hltv.org/teamlogo/Mzhpx1A4I2IitU0VrUJWuO.png?ixlib=java-2.1.0&amp;s=e4045d0a4495af7b65ca120143d5bb2e" class="team-logo" title="STMN"></div>
        </td>
        <td class="result-score"><span class="score-won">2</span> - <span class="score-lost">0</span></td>
        <td class="team-cell">
          <div class="line-align team2"><img alt="Loto" src="https://img-cdn.hltv.org/teamlogo/5Cn_5Y3SLF__fKg2kESiXO.png?ixlib=java-2.1.0&amp;s=a222e13bab4dae988ad4c887d93590bd" class="team-logo" title="Loto">
            <div class="team ">Loto</div>
          </div>
        </td>
        <td class="event"><img alt="FiReLEAGUE Latin Power - BLAST Premier Qualifier" src="https://static.hltv.org/images/eventLogos/5620.png" class="event-logo smartphone-only" title="FiReLEAGUE Latin Power - BLAST Premier Qualifier"><span class="event-name">FiReLEAGUE Latin Power - BLAST Premier Qualifier</span></td>
        <td class="star-cell">
          <div class="map-text">bo3</div>
        </td>
      </tr>
    </tbody></table>
  </div>
</a></div>
"#,
        );

        assert_eq!(
            MatchResult::from_element_ref(get_elem(&html, "div")),
            Ok(MatchResult {
                team1: "STMN".into(),
                team2: "Loto".into(),
                result: (2, 0),
                link:
                    "/matches/2345169/stmn-vs-loto-fireleague-latin-power-blast-premier-qualifier"
                        .into(),
                event: "FiReLEAGUE Latin Power - BLAST Premier Qualifier".into(),
                map: "bo3".into(),
                stars: 0,
            })
        )
    }

    #[test]
    fn parse_match_result_no_scores() {
        let html = Html::parse_fragment(
            r#"
<div class="result-con " data-zonedgrouping-entry-unix="1604796087000"><a href="/matches/2345169/stmn-vs-loto-fireleague-latin-power-blast-premier-qualifier" class="a-reset">
  <div class="result">
    <table>
      <tbody><tr>
        <td class="team-cell">
          <div class="line-align team1">
            <div class="team team-won">STMN</div>
ttps://img-cdn.hltv.org/teamlogo/Mzhpx1A4I2IitU0VrUJWuO.png?ixlib=java-2.1.0&amp;s=e4045d0a4495af7b65ca120143d5bb2e" class="team-logo" title="STMN"></div>
        </td>
        <td class="team-cell">
          <div class="line-align team2"><img alt="Loto" src="https://img-cdn.hltv.org/teamlogo/5Cn_5Y3SLF__fKg2kESiXO.png?ixlib=java-2.1.0&amp;s=a222e13bab4dae988ad4c887d93590bd" class="team-logo" title="Loto">
            <div class="team ">Loto</div>
          </div>
        </td>
        <td class="event"><img alt="FiReLEAGUE Latin Power - BLAST Premier Qualifier" src="https://static.hltv.org/images/eventLogos/5620.png" class="event-logo smartphone-only" title="FiReLEAGUE Latin Power - BLAST Premier Qualifier"><span class="event-name">FiReLEAGUE Latin Power - BLAST Premier Qualifier</span></td>
        <td class="star-cell">
          <div class="map-text">bo3</div>
        </td>
      </tr>
    </tbody></table>
  </div>
</a></div>
"#,
        );

        assert!(MatchResult::from_element_ref(get_elem(&html, "div")).is_err())
    }

    #[test]
    fn parse_matches_results() {
        let html = Html::parse_fragment(
            r#"
<html>
<div class="content">
<div class="results-holder">
  <div class="results-all" data-zonedgrouping-headline-format="'Results for' MMMM do y" data-zonedgrouping-headline-classes="standard-headline" data-zonedgrouping-group-classes="results-sublist">
    <div class="results-sublist">
      <span class="standard-headline">Results for November 8th 2020</span>
      <div class="result-con " data-zonedgrouping-entry-unix="1604796087000">
        <a href="/matches/2345169/stmn-vs-loto-fireleague-latin-power-blast-premier-qualifier" class="a-reset">
          <div class="result">
            <table>
              <tbody><tr>
                <td class="team-cell">
                  <div class="line-align team1">
                    <div class="team team-won">STMN</div>
                    <img alt="STMN" src="https://img-cdn.hltv.org/teamlogo/Mzhpx1A4I2IitU0VrUJWuO.png?ixlib=java-2.1.0&amp;s=e4045d0a4495af7b65ca120143d5bb2e" class="team-logo" title="STMN"></div>
                </td>
                <td class="result-score"><span class="score-won">2</span> - <span class="score-lost">0</span></td>
                <td class="team-cell">
                  <div class="line-align team2"><img alt="Loto" src="https://img-cdn.hltv.org/teamlogo/5Cn_5Y3SLF__fKg2kESiXO.png?ixlib=java-2.1.0&amp;s=a222e13bab4dae988ad4c887d93590bd" class="team-logo" title="Loto">
                    <div class="team ">Loto</div>
                  </div>
                </td>
                <td class="event"><img alt="FiReLEAGUE Latin Power - BLAST Premier Qualifier" src="https://static.hltv.org/images/eventLogos/5620.png" class="event-logo smartphone-only" title="FiReLEAGUE Latin Power - BLAST Premier Qualifier"><span class="event-name">FiReLEAGUE Latin Power - BLAST Premier Qualifier</span></td>
                <td class="star-cell">
                  <div class="map-text">bo3</div>
                </td>
              </tr>
              </tbody></table>
          </div>
        </a>
      </div>
      <div class="result-con " data-zonedgrouping-entry-unix="1604789807000">
        <a href="/matches/2345196/new-england-whalers-vs-chaos-iem-beijing-haidian-2020-north-america" class="a-reset">
          <div class="result">
            <table>
              <tbody><tr>
                <td class="team-cell">
                  <div class="line-align team1">
                    <div class="team ">New England Whalers</div>
                    <img alt="New England Whalers" src="https://img-cdn.hltv.org/teamlogo/Mo4qHDBnhzw2kVdAW-C6mD.png?ixlib=java-2.1.0&amp;s=9e6e7e8eb0f8d321ce84dd671e46b56b" class="team-logo" title="New England Whalers"></div>
                </td>
                <td class="result-score"><span class="score-lost">0</span> - <span class="score-won">2</span></td>
                <td class="team-cell">
                  <div class="line-align team2"><img alt="Chaos" src="https://img-cdn.hltv.org/teamlogo/Un_Sex9GVHDDkQof43bYNm.svg?ixlib=java-2.1.0&amp;s=3fca826b68536d844e3ffad96913cb06" class="team-logo" title="Chaos">
                    <div class="team team-won">Chaos</div>
                  </div>
                </td>
                <td class="event"><img alt="IEM Beijing-Haidian 2020 North America" src="https://static.hltv.org/images/eventLogos/5525.png" class="event-logo smartphone-only" title="IEM Beijing-Haidian 2020 North America"><span class="event-name">IEM Beijing-Haidian 2020 North America</span></td>
                <td class="star-cell">
                  <div class="map-and-stars">
                    <div class="stars"><i class="fa fa-star star"></i></div>
                    <div class="map map-text">bo3</div>
                  </div>
                </td>
              </tr>
              </tbody></table>
          </div>
        </a>
      </div>
    </div>
    <div class="results-sublist">
      <span class="standard-headline">Results for November 7th 2020</span>
      <div class="result-con " data-zonedgrouping-entry-unix="1604781723000">
        <a href="/matches/2345190/complexity-vs-fnatic-iem-beijing-haidian-2020-europe" class="a-reset">
          <div class="result">
            <table>
              <tbody><tr>
                <td class="team-cell">
                  <div class="line-align team1">
                    <div class="team team-won">Complexity</div>
                    <img alt="Complexity" src="https://img-cdn.hltv.org/teamlogo/R0CzydpyX02BnkAYhy3I89.svg?ixlib=java-2.1.0&amp;s=8c5833d6069ef924fdbb2e220fefea00" class="team-logo day-only" title="Complexity"><img alt="Complexity" src="https://img-cdn.hltv.org/teamlogo/0-i_bEjrf3v4eYqaG0Bix7.svg?ixlib=java-2.1.0&amp;s=4eecbec277f018772a9b92c22da1a459" class="team-logo night-only" title="Complexity"></div>
                </td>
                <td class="result-score"><span class="score-won">2</span> - <span class="score-lost">0</span></td>
                <td class="team-cell">
                  <div class="line-align team2"><img alt="fnatic" src="https://img-cdn.hltv.org/teamlogo/dLtWEdSV58lIX1amAFggy0.svg?ixlib=java-2.1.0&amp;s=f24d0a7b3ef24ed57184a51d35202b4e" class="team-logo" title="fnatic">
                    <div class="team ">fnatic</div>
                  </div>
                </td>
                <td class="event"><img alt="IEM Beijing-Haidian 2020 Europe" src="https://static.hltv.org/images/eventLogos/5524.png" class="event-logo smartphone-only" title="IEM Beijing-Haidian 2020 Europe"><span class="event-name">IEM Beijing-Haidian 2020 Europe</span></td>
                <td class="star-cell">
                  <div class="map-and-stars">
                    <div class="stars"><i class="fa fa-star star"></i><i class="fa fa-star star"></i></div>
                    <div class="map map-text">bo3</div>
                  </div>
                </td>
              </tr>
              </tbody></table>
          </div>
        </a>
      </div>
      <div class="result-con " data-zonedgrouping-entry-unix="1604781457000">
        <a href="/matches/2345166/9z-vs-stmn-fireleague-latin-power-blast-premier-qualifier" class="a-reset">
          <div class="result">
            <table>
              <tbody><tr>
                <td class="team-cell">
                  <div class="line-align team1">
                    <div class="team team-won">9z</div>
                    <img alt="9z" src="https://img-cdn.hltv.org/teamlogo/lpPZKI1dJN8YknitEkuvK4.png?ixlib=java-2.1.0&amp;s=14d42f083af66c88bdca69bcb4538362" class="team-logo" title="9z"></div>
                </td>
                <td class="result-score"><span class="score-won">16</span> - <span class="score-lost">12</span></td>
                <td class="team-cell">
                  <div class="line-align team2"><img alt="STMN" src="https://img-cdn.hltv.org/teamlogo/Mzhpx1A4I2IitU0VrUJWuO.png?ixlib=java-2.1.0&amp;s=e4045d0a4495af7b65ca120143d5bb2e" class="team-logo" title="STMN">
                    <div class="team ">STMN</div>
                  </div>
                </td>
                <td class="event"><img alt="FiReLEAGUE Latin Power - BLAST Premier Qualifier" src="https://static.hltv.org/images/eventLogos/5620.png" class="event-logo smartphone-only" title="FiReLEAGUE Latin Power - BLAST Premier Qualifier"><span class="event-name">FiReLEAGUE Latin Power - BLAST Premier Qualifier</span></td>
                <td class="star-cell">
                  <div class="map-text">d2</div>
                </td>
              </tr>
              </tbody></table>
          </div>
        </a>
      </div>
      <div class="result-con " data-zonedgrouping-entry-unix="1604777663000">
        <a href="/matches/2345289/saw-vs-giants-master-league-portugal-vi" class="a-reset">
          <div class="result">
            <table>
              <tbody><tr>
                <td class="team-cell">
                  <div class="line-align team1">
                    <div class="team ">sAw</div>
                    <img alt="sAw" src="https://img-cdn.hltv.org/teamlogo/PUWg2rwPedcO6onf1jBIaC.png?ixlib=java-2.1.0&amp;s=95a226995991d35b74d75d193964230d" class="team-logo" title="sAw"></div>
                </td>
                <td class="result-score"><span class="score-lost">1</span> - <span class="score-won">2</span></td>
                <td class="team-cell">
                  <div class="line-align team2"><img alt="Giants" src="https://img-cdn.hltv.org/teamlogo/KYdRkHF4byzZ_e5fQpzBbf.png?ixlib=java-2.1.0&amp;s=5ddae2a752dd43f46f22975b4fedd9fd" class="team-logo" title="Giants">
                    <div class="team team-won">Giants</div>
                  </div>
                </td>
                <td class="event"><img alt="Master League Portugal VI" src="https://static.hltv.org/images/eventLogos/5628.png" class="event-logo smartphone-only" title="Master League Portugal VI"><span class="event-name">Master League Portugal VI</span></td>
                <td class="star-cell">
                  <div class="map-text">bo3</div>
                </td>
              </tr>
              </tbody></table>
          </div>
        </a>
      </div>
    </div>
  </div>
  <div class="pagination-component pagination-bottom"><span class="pagination-data">1 - 100 of 56650 </span><a class="pagination-prev"><i class=" fa fa-chevron-left pagination-left" aria-hidden="true"></i></a> <a href="/results?offset=100" class="pagination-next"><i class=" fa fa-chevron-right pagination-right" aria-hidden="true"></i></a></div>
  <span class="clearfix"></span>
</div>
</div>
</html>
"#,
        );

        let res = MatchesResults::from_html(&html).unwrap();
        assert_eq!(res.results.len(), 2);
        assert_eq!(res.results["Results for November 7th 2020"].len(), 3);
        assert_eq!(res.results["Results for November 8th 2020"].len(), 2);
    }

    #[test]
    fn parse_matches_results_err() {
        let html = Html::parse_fragment(
            r#"
<div class="results-holder">
  <div class="results-all" data-zonedgrouping-headline-format="'Results for' MMMM do y" data-zonedgrouping-headline-classes="standard-headline" data-zonedgrouping-group-classes="results-sublist">
    <div class="results-sublist">
      <div class="result-con " data-zonedgrouping-entry-unix="1604796087000">
        <a href="/matches/2345169/stmn-vs-loto-fireleague-latin-power-blast-premier-qualifier" class="a-reset">
          <div class="result">
            <table>
              <tbody><tr>
                <td class="team-cell">
                  <div class="line-align team1">
                    <div class="team team-won">STMN</div>
                    <img alt="STMN" src="https://img-cdn.hltv.org/teamlogo/Mzhpx1A4I2IitU0VrUJWuO.png?ixlib=java-2.1.0&amp;s=e4045d0a4495af7b65ca120143d5bb2e" class="team-logo" title="STMN"></div>
                </td>
                <td class="result-score"><span class="score-won">2</span> - <span class="score-lost">0</span></td>
                <td class="team-cell">
                  <div class="line-align team2"><img alt="Loto" src="https://img-cdn.hltv.org/teamlogo/5Cn_5Y3SLF__fKg2kESiXO.png?ixlib=java-2.1.0&amp;s=a222e13bab4dae988ad4c887d93590bd" class="team-logo" title="Loto">
                    <div class="team ">Loto</div>
                  </div>
                </td>
                <td class="event"><img alt="FiReLEAGUE Latin Power - BLAST Premier Qualifier" src="https://static.hltv.org/images/eventLogos/5620.png" class="event-logo smartphone-only" title="FiReLEAGUE Latin Power - BLAST Premier Qualifier"><span class="event-name">FiReLEAGUE Latin Power - BLAST Premier Qualifier</span></td>
                <td class="star-cell">
                  <div class="map-text">bo3</div>
                </td>
              </tr>
              </tbody></table>
          </div>
        </a>
      </div>
    </div>
  <div class="pagination-component pagination-bottom"><span class="pagination-data">1 - 100 of 56650 </span><a class="pagination-prev"><i class=" fa fa-chevron-left pagination-left" aria-hidden="true"></i></a> <a href="/results?offset=100" class="pagination-next"><i class=" fa fa-chevron-right pagination-right" aria-hidden="true"></i></a></div>
  <span class="clearfix"></span>
</div>
"#,
        );

        assert!(MatchesResults::from_html(&html).is_err());
    }
}
