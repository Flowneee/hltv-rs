use std::collections::HashMap;

use scraper::selector;
use scraper::{Html, Selector};

use crate::{
    api::{ElementRef, ElementRefExt},
    Error, NoneErrorExt, Result,
};

#[derive(Debug, PartialEq)]
pub enum UpcomingMatchTeam {
    Name(String),
    Tbd(String),
}

impl UpcomingMatchTeam {
    // TODO: doc
    pub fn from_element_ref(element: ElementRef) -> Result<Self> {
        if let Some(name_elem) = element.select_one(".matchTeamName")? {
            Ok(Self::Name(name_elem.text2()))
        } else {
            Ok(Self::Tbd(
                element
                    .select_one(".team")?
                    .hltv_parse_err("Failed to find TBD team info")?
                    .text2(),
            ))
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum UpcomingMatchTeams {
    Empty {
        description: String,
    },
    Teams {
        team1: UpcomingMatchTeam,
        team2: UpcomingMatchTeam,
        event: String,
    },
}

#[derive(Debug, PartialEq)]
pub struct UpcomingMatch {
    teams: UpcomingMatchTeams,
    time: String,  // TODO: normal time type
    rating: usize, // num of stars
    meta: String,  // format
    link: String,
}

impl UpcomingMatch {
    // TODO: doc
    pub fn from_element_ref(element: ElementRef) -> Result<Self> {
        let link = element
            .select_one("a")?
            .hltv_parse_err("No 'a' element with link to upcoming match")?
            .value()
            .attr("href")
            .hltv_parse_err("No href to upcoming match")?
            .into();
        let time = element
            .select_one(".matchInfo>.matchTime")?
            .hltv_parse_err("Failed to find match time")?
            .text2();
        let rating = element
            .select(&Selector::parse(".matchInfo>.matchRating>.fa-star")?)
            .filter(|elem| !elem.value().classes().any(|v| v == "faded"))
            .count();
        let meta = element
            .select_one(".matchInfo>.matchMeta")?
            .hltv_parse_err("Failed to get match meta (format)")?
            .text2();

        let teams = if let Some(elem) = element.select_one(".matchInfoEmpty>span")? {
            UpcomingMatchTeams::Empty {
                description: elem.text2(),
            }
        } else {
            let event = element
                .select_one(".matchEvent>.matchEventName")?
                .hltv_parse_err("Failed to find event")?
                .text2();
            let team1 = UpcomingMatchTeam::from_element_ref(
                element
                    .select_one(".matchTeams>.team1")?
                    .hltv_parse_err("Failed to find team1 element")?,
            )?;
            let team2 = UpcomingMatchTeam::from_element_ref(
                element
                    .select_one(".matchTeams>.team2")?
                    .hltv_parse_err("Failed to find team2 element")?,
            )?;

            UpcomingMatchTeams::Teams {
                team1,
                team2,
                event,
            }
        };

        Ok(Self {
            teams,
            time,
            rating,
            meta,
            link,
        })
    }
}

pub struct UpcomingMatches {
    pub results: HashMap<String, Vec<UpcomingMatch>>,
}

impl UpcomingMatches {
    // TODO: doc
    pub fn from_html(document: &Html) -> Result<Self> {
        document
            .select(&Selector::parse(
                ".upcomingMatchesContainer>div>div.upcomingMatchesSection",
            )?)
            .map(|element| {
                let day = element
                    .select_one(".matchDayHeadline")? // Seems like it changes from div to span in JS
                    .hltv_parse_err("Failed to find match day headline")?
                    .text2();

                let results = element
                    .select(&Selector::parse("div.upcomingMatch")?)
                    .map(|x| UpcomingMatch::from_element_ref(x))
                    .collect::<Result<Vec<_>>>()?;
                Ok((day, results))
            })
            .collect::<Result<HashMap<_, _>>>()
            .map(|results| Self { results })
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
    fn parse_upcoming_match_team() {
        let html = Html::parse_fragment(
            r#"
<div class="matchTeam team1">
  <div class="matchTeamLogoContainer"><img alt="NiP" src="https://img-cdn.hltv.org/teamlogo/BSmtTpoXWe5bkSQ1Xk9bBQ.svg?ixlib=java-2.1.0&amp;s=a0edf9bc3edb8680461c858fa21fe7fe" class="matchTeamLogo" title="NiP"></div>
  <div class="matchTeamName text-ellipsis">NiP</div>
</div>
"#,
        );

        assert_eq!(
            UpcomingMatchTeam::from_element_ref(get_elem(&html, "div")),
            Ok(UpcomingMatchTeam::Name("NiP".into()))
        );
    }

    #[test]
    fn parse_upcoming_match_team_tbd() {
        let html = Html::parse_fragment(
            r#"
<div class="matchTeam team2">
        <div class="team text-ellipsis">ENCE/BIG winner</div>
      </div>
"#,
        );

        assert_eq!(
            UpcomingMatchTeam::from_element_ref(get_elem(&html, "div")),
            Ok(UpcomingMatchTeam::Tbd("ENCE/BIG winner".into()))
        );
    }

    #[test]
    fn parse_upcoming_match_info() {
        let html = Html::parse_fragment(
            r#"
<div class="upcomingMatch removeBackground" data-zonedgrouping-entry-unix="1605547800000" stars="1" lan="false" filteraslive="false" team1="4411">
  <a href="/matches/2345214/nip-vs-ence-big-winner-iem-beijing-haidian-2020-europe" class="match a-reset">
    <div class="matchInfo">
      <div class="matchTime" data-time-format="HH:mm" data-unix="1605547800000">20:30</div>
      <div class="matchRating"><i class="fa fa-star"></i><i class="fa fa-star faded"></i><i class="fa fa-star faded"></i><i class="fa fa-star faded"></i><i class="fa fa-star faded"></i></div>
      <div class="matchMeta">bo3</div>
    </div>
    <div class="matchTeams text-ellipsis">
      <div class="matchTeam team1">
        <div class="matchTeamLogoContainer"><img alt="NiP" src="https://img-cdn.hltv.org/teamlogo/BSmtTpoXWe5bkSQ1Xk9bBQ.svg?ixlib=java-2.1.0&amp;s=a0edf9bc3edb8680461c858fa21fe7fe" class="matchTeamLogo" title="NiP"></div>
        <div class="matchTeamName text-ellipsis">NiP</div>
      </div>
      <div class="matchTeam team2">
        <div class="team text-ellipsis">ENCE/BIG winner</div>
      </div>
    </div>
    <div class="matchEvent">
      <div class="matchEventLogoContainer"><img alt="IEM Beijing-Haidian 2020 Europe" src="https://static.hltv.org/images/eventLogos/5524.png" class="matchEventLogo" title="IEM Beijing-Haidian 2020 Europe"></div>
      <div class="matchEventName gtSmartphone-only">IEM Beijing-Haidian 2020 Europe</div>
    </div>
</a>s/2345214/nip-vs-ence-big-winner-iem-beijing-haidian-2020-europe" class="matchAnalytics" title="Analytics">
      <div class="analyticsLink"><i class="fa fa-bar-chart"></i><span class="gtSmartphone-only">A</span></div>
  </a>
</div>
"#,
        );

        assert_eq!(
            UpcomingMatch::from_element_ref(get_elem(&html, "div")),
            Ok(UpcomingMatch {
                teams: UpcomingMatchTeams::Teams {
                    team1: UpcomingMatchTeam::Name("NiP".into()),
                    team2: UpcomingMatchTeam::Tbd("ENCE/BIG winner".into()),
                    event: "IEM Beijing-Haidian 2020 Europe".into(),
                },
                time: "20:30".into(),
                rating: 1,
                meta: "bo3".into(),
                link: "/matches/2345214/nip-vs-ence-big-winner-iem-beijing-haidian-2020-europe"
                    .into(),
            })
        );
    }

    #[test]
    fn parse_day_results() {
        let html = Html::parse_fragment(
            r#"
<hltv>
<div class="standardPageGrid">
  <div class="mainContent">
    <div class="liveMatchesSection">
      <div class="headline-flex no-shadow">
        <h2 class="upcoming-headline">Live CS:GO matches</h2>
        <div><i class="star-filter-btn matchpage-star-unselected fa fa-star-o" title="Enable match filter"></i><i class="star-filter-btn matchpage-star-selected fa fa-star" title="Disable match filter" style="display: none;"></i></div>
      </div>
      <div class="mach-filter-wrapper">
        <div class="match-filter">
          <div class="match-filter-box">
            <div class="filter-main-content"><a href="/matches" class="filter-button-link">
                <div class="filter-button  selected">
                  <div class="icon"><img src="img/static/gfx/cs_icon_active_day.png" class="day-only custom-icon"><img src="img/static/gfx/cs_icon_active_night.png" class="night-only custom-icon"></div>
                  <div class="button-title">All matches</div>
                </div>
              </a><a href="/matches?predefinedFilter=top_tier" class="filter-button-link">
                <div class="filter-button  ">
                  <div class="icon"><i class="fa fa-star"></i></div>
                  <div class="button-title">Top tier</div>
                </div>
              </a><a href="/matches?predefinedFilter=lan_only" class="filter-button-link">
                <div class="filter-button  ">
                  <div class="icon"><i class="fa fa-desktop"></i></div>
                  <div class="button-title">LAN</div>
                </div>
              </a>
              <div class="filter-button custom ">
                <div class="icon"><i class="fa fa-trophy"></i></div>
                <div class="button-title">Event</div>
              </div>
              <div class="extraSpacer smartphone-only"></div>
            </div>
          </div>
          <div class="filter-custom-content ">
            <div class="event-type">
              <div class="custom-content-header">Event type</div>
              <div><a href="/matches?eventType=All" class="event-filter-link active">All</a><a href="/matches?eventType=Lan" class="event-filter-link ">Lan</a><a href="/matches?eventType=Online" class="event-filter-link ">Online</a></div>
            </div>
            <div class="event">
              <div class="custom-content-header">Event</div>
              <div class="events-container"><a href="/matches?event=5602" class="filter-button-link">
                  <div class="event-button  tooltip-parent"><img alt="BLAST Premier Spring Groups 2021" src="https://img-cdn.hltv.org/eventlogo/O8dyTstiXZp1wPIcOGi_GC.png?ixlib=java-2.1.0&amp;s=c414e930b554c2cba8f1098fa3619d51" class="event-logo" title="">
                    <div class="featured-event-tooltip">
                      <div class="featured-event-tooltip-content">BLAST Premier Spring Groups 2021</div>
                    </div>
                  </div>
                </a><a href="/matches?event=5671" class="filter-button-link">
                  <div class="event-button  tooltip-parent"><img alt="IEM Katowice 2021 Play-In" src="https://img-cdn.hltv.org/eventlogo/_a7BE4dbvKVdBFTDaTPULq.png?ixlib=java-2.1.0&amp;s=f83842b885fe6bf00fc8e75a501ee955" class="event-logo" title="">
                    <div class="featured-event-tooltip">
                      <div class="featured-event-tooltip-content">IEM Katowice 2021 Play-In</div>
                    </div>
                  </div>
                </a><a href="/matches?event=5701" class="filter-button-link">
                  <div class="event-button  tooltip-parent"><img alt="ESEA Premier Season 36 Europe" src="https://img-cdn.hltv.org/eventlogo/b75aNG0i4UVPNQHX_Tq-Zq.png?ixlib=java-2.1.0&amp;s=a41982a53b2a3d56ca657c6f6335259d" class="event-logo" title="">
                    <div class="featured-event-tooltip">
                      <div class="featured-event-tooltip-content">ESEA Premier Season 36 Europe</div>
                    </div>
                  </div>
                </a><a href="/matches?event=5704" class="filter-button-link">
                  <div class="event-button  tooltip-parent"><img alt="ESEA Premier Season 36 North America" src="https://img-cdn.hltv.org/eventlogo/Fu5w0q5fm_JedUt4Trlx39.png?ixlib=java-2.1.0&amp;s=116ecf0d78dec2bcafafa861f8398c93" class="event-logo" title="">
                    <div class="featured-event-tooltip">
                      <div class="featured-event-tooltip-content">ESEA Premier Season 36 North America</div>
                    </div>
                  </div>
                </a>
                <div class="event-custom-container">
                  <div class="event-button expand-event-button event-filter-btn ">...</div>
                  <div class="event-filter-popup"><a href="/matches?event=5727" class="filter-button-link event-row">
                      <div class="event-img"><img alt="European Development Championship 2" src="https://img-cdn.hltv.org/eventlogo/fUUnE-XkcPKbEohf3bvgsl.png?ixlib=java-2.1.0&amp;s=3c5518b338c4c4c22757a29051dd2bcb" title="European Development Championship 2"></div>
                      <div class="event-name">European Development Championship 2</div>
heckbox" class="event-checkbox">
                      <div class="container-overlay"></div>
                    </a><a href="/matches?event=5725" class="filter-button-link event-row">
                      <div class="event-img"><img alt="WESG 2021 LatAm South" src="https://img-cdn.hltv.org/eventlogo/DvZDZiwawcnDV1AcWyNn4m.png?ixlib=java-2.1.0&amp;s=fcefa5d8451b27327772fb9963037e69" title="WESG 2021 LatAm South"></div>
                      <div class="event-name">WESG 2021 LatAm South</div>
heckbox" class="event-checkbox">
                      <div class="container-overlay"></div>
                    </a></div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
      <div class="liveMatchesContainer">
        <div class="liveMatches" data-scorebot-url="https://scorebot-lb.hltv.org">
          <div class="liveMatch-container" data-scorebot-id="2346342" data-team1-id="4608" data-team2-id="6667" data-hide-map-before-live="true" data-maps="Inferno,Nuke,Train" stars="2" lan="false" filteraslive="true" team1="4608" team2="6667">
            <div class="liveMatch" data-livescore-match="2346342"><a href="/matches/2346342/natus-vincere-vs-faze-blast-premier-spring-groups-2021" class="match a-reset">
                <div class="matchInfo">
                  <div class="matchTime matchLive">LIVE</div>
                  <div class="matchRating matchLive"><i class="fa fa-star"></i><i class="fa fa-star"></i><i class="fa fa-star faded"></i><i class="fa fa-star faded"></i><i class="fa fa-star faded"></i></div>
                  <div class="matchMeta">bo3</div>
                </div>
                <div class="matchTeams text-ellipsis">
                  <div class="matchTeam">
                    <div class="matchTeamLogoContainer"><img alt="Natus Vincere" src="https://img-cdn.hltv.org/teamlogo/kixzGZIb9IYAAv-1vGrGev.svg?ixlib=java-2.1.0&amp;s=8f9986a391fcb1adfbfff021b824a937" class="matchTeamLogo" title="Natus Vincere"></div>
                    <div class="matchTeamName text-ellipsis">Natus Vincere</div>
                    <div class="matchTeamScore"><span class="currentMapScore leading" data-livescore-current-map-score="" data-livescore-team="4608"> 5</span><span class="mapScore"> (<span data-livescore-maps-won-for="" data-livescore-team="4608" class="none">0</span>)</span></div>
                  </div>
                  <div class="matchTeam">
                    <div class="matchTeamLogoContainer"><img alt="FaZe" src="https://img-cdn.hltv.org/teamlogo/SMhzsxzbkIrgqCOOKGRXlW.svg?ixlib=java-2.1.0&amp;s=e6a9ce0345c7d703e5eaac14307f69aa" class="matchTeamLogo" title="FaZe"></div>
                    <div class="matchTeamName text-ellipsis">FaZe</div>
                    <div class="matchTeamScore"><span class="currentMapScore trailing" data-livescore-current-map-score="" data-livescore-team="6667"> 4</span><span class="mapScore"> (<span data-livescore-maps-won-for="" data-livescore-team="6667" class="none">0</span>)</span></div>
                  </div>
                </div>
                <div class="matchEvent ">
                  <div class="matchEventLogoContainer"><img alt="BLAST Premier Spring Groups 2021" src="https://img-cdn.hltv.org/eventlogo/O8dyTstiXZp1wPIcOGi_GC.png?ixlib=java-2.1.0&amp;s=c414e930b554c2cba8f1098fa3619d51" class="matchEventLogo" title="BLAST Premier Spring Groups 2021"></div>
                  <div class="matchEventName gtSmartphone-only">BLAST Premier Spring Groups 2021</div>
                </div>
betting/analytics/2346342/natus-vincere-vs-faze-blast-premier-spring-groups-2021" class="matchAnalytics" title="Analytics">
                  <div class="analyticsLink"><i class="fa fa-bar-chart"></i><span class="gtSmartphone-only">A</span></div>
                </a></div>
            <div class="scorebot-container" id="matchScorebotId2346342"></div>
            <div class="expand-match-btn">Expand</div>
          </div>
          <div class="liveMatch-container" data-scorebot-id="2346452" data-team1-id="8963" data-team2-id="8135" data-hide-map-before-live="true" data-maps="Dust2,Nuke,Inferno" stars="0" lan="false" filteraslive="true" team1="8963" team2="8135">
            <div class="liveMatch" data-livescore-match="2346452"><a href="/matches/2346452/lyngby-vikings-vs-forze-european-development-championship-2" class="match a-reset">
                <div class="matchInfo">
                  <div class="matchTime matchLive">LIVE</div>
                  <div class="matchRating matchLive"><i class="fa fa-star faded"></i><i class="fa fa-star faded"></i><i class="fa fa-star faded"></i><i class="fa fa-star faded"></i><i class="fa fa-star faded"></i></div>
                  <div class="matchMeta">bo3</div>
                </div>
                <div class="matchTeams text-ellipsis">
                  <div class="matchTeam">
                    <div class="matchTeamLogoContainer"><img alt="Lyngby Vikings" src="https://img-cdn.hltv.org/teamlogo/-VPKbzCklmJ9QntObIRT7u.svg?ixlib=java-2.1.0&amp;s=1eff5dfa200b8f6cf286135a131ccd94" class="matchTeamLogo" title="Lyngby Vikings"></div>
                    <div class="matchTeamName text-ellipsis">Lyngby Vikings</div>
                    <div class="matchTeamScore"><span class="currentMapScore trailing" data-livescore-current-map-score="" data-livescore-team="8963"> 7</span><span class="mapScore"> (<span data-livescore-maps-won-for="" data-livescore-team="8963" class="trailing">0</span>)</span></div>
                  </div>
                  <div class="matchTeam">
                    <div class="matchTeamLogoContainer"><img alt="forZe" src="https://img-cdn.hltv.org/teamlogo/Qnpb1nBNLJUCyf4fRMFbzr.svg?ixlib=java-2.1.0&amp;s=a798b973c429361844ee174e07ae2401" class="matchTeamLogo" title="forZe"></div>
                    <div class="matchTeamName text-ellipsis">forZe</div>
                    <div class="matchTeamScore"><span class="currentMapScore leading" data-livescore-current-map-score="" data-livescore-team="8135"> 9</span><span class="mapScore"> (<span data-livescore-maps-won-for="" data-livescore-team="8135" class="leading">1</span>)</span></div>
                  </div>
                </div>
                <div class="matchEvent ">
                  <div class="matchEventLogoContainer"><img alt="European Development Championship 2" src="https://img-cdn.hltv.org/eventlogo/fUUnE-XkcPKbEohf3bvgsl.png?ixlib=java-2.1.0&amp;s=3c5518b338c4c4c22757a29051dd2bcb" class="matchEventLogo" title="European Development Championship 2"></div>
                  <div class="matchEventName gtSmartphone-only">European Development Championship 2</div>
                </div>
betting/analytics/2346452/lyngby-vikings-vs-forze-european-development-championship-2" class="matchAnalytics" title="Analytics">
                  <div class="analyticsLink"><i class="fa fa-bar-chart"></i><span class="gtSmartphone-only">A</span></div>
                </a></div>
            <div class="scorebot-container" id="matchScorebotId2346452"></div>
            <div class="expand-match-btn">Expand</div>
          </div>
        </div>
      </div>
    </div>
    <div class="headline-flex">
      <h1 class="upcoming-headline">Upcoming CS:GO matches</h1>
    </div>
    <div class="upcomingMatchesWrapper">
      <div class="upcomingMatchesContainer">
        <div class="" data-zonedgrouping-headline-format="EEEE - yyyy-MM-dd" data-zonedgrouping-headline-classes="matchDayHeadline" data-zonedgrouping-group-classes="upcomingMatchesSection">
          <div class="upcomingMatchesSection">
            <div class="matchDayHeadline">Saturday - 2021-02-13</div>
            <div class="upcomingMatch removeBackground" data-zonedgrouping-entry-unix="1613241000000" stars="2" lan="false" filteraslive="false" team1="9215" team2="5973">
              <a href="/matches/2346343/mibr-vs-liquid-blast-premier-spring-groups-2021" class="match a-reset">
                <div class="matchInfo">
                  <div class="matchTime" data-time-format="HH:mm" data-unix="1613241000000">21:30</div>
                  <div class="matchRating"><i class="fa fa-star"></i><i class="fa fa-star"></i><i class="fa fa-star faded"></i><i class="fa fa-star faded"></i><i class="fa fa-star faded"></i></div>
                  <div class="matchMeta">bo3</div>
                </div>
                <div class="matchTeams text-ellipsis">
                  <div class="matchTeam team1">
                    <div class="matchTeamLogoContainer"><img alt="MIBR" src="https://img-cdn.hltv.org/teamlogo/sVnH-oAf1J5TnMwoY4cxUC.png?ixlib=java-2.1.0&amp;w=50&amp;s=b0ef463fa0f1638bce72a89590fbaddf" class="matchTeamLogo day-only" title="MIBR"><img alt="MIBR" src="https://img-cdn.hltv.org/teamlogo/m_JQ624LNFHWiUY-25uuaE.png?ixlib=java-2.1.0&amp;w=50&amp;s=80a1e479dd1b15b974d3e2d5588763af" class="matchTeamLogo night-only" title="MIBR"></div>
                    <div class="matchTeamName text-ellipsis">MIBR</div>
                  </div>
                  <div class="matchTeam team2">
                    <div class="matchTeamLogoContainer"><img alt="Liquid" src="https://img-cdn.hltv.org/teamlogo/JMeLLbWKCIEJrmfPaqOz4O.svg?ixlib=java-2.1.0&amp;s=c02caf90234d3a3ebac074c84ba1ea62" class="matchTeamLogo" title="Liquid"></div>
                    <div class="matchTeamName text-ellipsis">Liquid</div>
                  </div>
                </div>
                <div class="matchEvent">
                  <div class="matchEventLogoContainer"><img alt="BLAST Premier Spring Groups 2021" src="https://img-cdn.hltv.org/eventlogo/O8dyTstiXZp1wPIcOGi_GC.png?ixlib=java-2.1.0&amp;s=c414e930b554c2cba8f1098fa3619d51" class="matchEventLogo" title="BLAST Premier Spring Groups 2021"></div>
                  <div class="matchEventName gtSmartphone-only">BLAST Premier Spring Groups 2021</div>
                </div>
betting/analytics/2346343/mibr-vs-liquid-blast-premier-spring-groups-2021" class="matchAnalytics" title="Analytics">
                  <div class="analyticsLink"><i class="fa fa-bar-chart"></i><span class="gtSmartphone-only">A</span></div>
                </a></div></div><div class="upcomingMatchesSection"><div class="matchDayHeadline">Sunday - 2021-02-14</div><div class="upcomingMatch removeBackground" data-zonedgrouping-entry-unix="1613316600000" stars="2" lan="false" filteraslive="false"><a href="/matches/2346344/blast-premier-spring-groups-2021-group-c-consolidation-final-blast-premier-spring-groups-2021" class="match a-reset">
                <div class="matchInfo">
                  <div class="matchTime" data-time-format="HH:mm" data-unix="1613316600000">18:30</div>
                  <div class="matchRating"><i class="fa fa-star"></i><i class="fa fa-star"></i><i class="fa fa-star faded"></i><i class="fa fa-star faded"></i><i class="fa fa-star faded"></i></div>
                  <div class="matchMeta">bo3</div>
                </div>
                <div class="matchInfoEmpty"><span class="line-clamp-3">BLAST Premier Spring Groups 2021 - Group C Consolidation Final</span></div>
betting/analytics/2346344/blast-premier-spring-groups-2021-group-c-consolidation-final-blast-premier-spring-groups-2021" class="matchAnalytics" title="Analytics">
                  <div class="analyticsLink"><i class="fa fa-bar-chart"></i><span class="gtSmartphone-only">A</span></div>
                </a></div><div class="upcomingMatch removeBackground oddRowBgColor" data-zonedgrouping-entry-unix="1613331000000" stars="2" lan="false" filteraslive="false"><a href="/matches/2346345/blast-premier-spring-groups-2021-group-c-final-blast-premier-spring-groups-2021" class="match a-reset">
                <div class="matchInfo">
                  <div class="matchTime" data-time-format="HH:mm" data-unix="1613331000000">22:30</div>
                  <div class="matchRating"><i class="fa fa-star"></i><i class="fa fa-star"></i><i class="fa fa-star faded"></i><i class="fa fa-star faded"></i><i class="fa fa-star faded"></i></div>
                  <div class="matchMeta">bo3</div>
                </div>
                <div class="matchInfoEmpty"><span class="line-clamp-3">BLAST Premier Spring Groups 2021 - Group C Final</span></div>
betting/analytics/2346345/blast-premier-spring-groups-2021-group-c-final-blast-premier-spring-groups-2021" class="matchAnalytics" title="Analytics">
                  <div class="analyticsLink"><i class="fa fa-bar-chart"></i><span class="gtSmartphone-only">A</span></div>
                </a></div></div><div class="upcomingMatchesSection"><div class="matchDayHeadline">Thursday - 2021-03-04</div><div class="upcomingMatch removeBackground" data-zonedgrouping-entry-unix="1614880800000" stars="0" lan="false" filteraslive="false" team1="9863" team2="6978"><a href="/matches/2346506/fate-vs-singularity-esea-premier-season-36-europe" class="match a-reset">
                <div class="matchInfo">
                  <div class="matchTime" data-time-format="HH:mm" data-unix="1614880800000">21:00</div>
                  <div class="matchRating"><i class="fa fa-star faded"></i><i class="fa fa-star faded"></i><i class="fa fa-star faded"></i><i class="fa fa-star faded"></i><i class="fa fa-star faded"></i></div>
                  <div class="matchMeta">bo3</div>
                </div>
                <div class="matchTeams text-ellipsis">
                  <div class="matchTeam team1">
                    <div class="matchTeamLogoContainer"><img alt="FATE" src="https://img-cdn.hltv.org/teamlogo/N2kh0YzH5DEk2tnOUAXtx6.png?ixlib=java-2.1.0&amp;w=50&amp;s=9f0542fa00872a3f0bf449aa502ef364" class="matchTeamLogo" title="FATE"></div>
                    <div class="matchTeamName text-ellipsis">FATE</div>
                  </div>
                  <div class="matchTeam team2">
                    <div class="matchTeamLogoContainer"><img alt="Singularity" src="https://img-cdn.hltv.org/teamlogo/C1Nyy0ZcMxR_iUlJFLtUW7.svg?ixlib=java-2.1.0&amp;s=d18bb6c2a85a032f371a14a10207f183" class="matchTeamLogo" title="Singularity"></div>
                    <div class="matchTeamName text-ellipsis">Singularity</div>
                  </div>
                </div>
                <div class="matchEvent">
                  <div class="matchEventLogoContainer"><img alt="ESEA Premier Season 36 Europe" src="https://img-cdn.hltv.org/eventlogo/b75aNG0i4UVPNQHX_Tq-Zq.png?ixlib=java-2.1.0&amp;s=a41982a53b2a3d56ca657c6f6335259d" class="matchEventLogo" title="ESEA Premier Season 36 Europe"></div>
                  <div class="matchEventName gtSmartphone-only">ESEA Premier Season 36 Europe</div>
                </div>
betting/analytics/2346506/fate-vs-singularity-esea-premier-season-36-europe" class="matchAnalytics" title="Analytics">
                  <div class="analyticsLink"><i class="fa fa-bar-chart"></i><span class="gtSmartphone-only">A</span></div>
                </a></div></div></div>
        <div class="match-filter-empty-warning newMatchesEmptystateContainer " style="display: none;">Matches with selected filter are hidden. Disable your match filter to view all matches.
          <div class="match-filter-warning-disable-container"><span class="match-filter-warning-disable" style="display: none;"><i class="fa fa-star"></i>Disable starfilter</span></div>
        </div>
      </div>
    </div>
  </div>
</div>
</hltv>
"#,
        );

        let res = UpcomingMatches::from_html(&html).unwrap();
        assert_eq!(res.results.len(), 3);
        assert_eq!(res.results["Saturday - 2021-02-13"].len(), 1);
        assert_eq!(res.results["Sunday - 2021-02-14"].len(), 2);
    }
}
