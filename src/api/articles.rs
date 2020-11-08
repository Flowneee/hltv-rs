use scraper::{Html, Selector};

use super::{ElementRef, ElementRefExt};
use crate::{NoneErrorExt, Result};

#[derive(Debug, PartialEq)]
pub struct ArticleBrief {
    pub name: String,
    pub path: String,
    pub when: String,
    pub comments_num: String,
}

impl ArticleBrief {
    // TODO: doc
    pub fn from_element_ref(element: ElementRef) -> Result<Self> {
        let path = element
            .value()
            .attr("href")
            .hltv_parse_err("Cannot find href for news brief")?
            .to_string();
        let name = element
            .select_one("div.newstext")?
            .hltv_parse_err("Cannot find name for news brief")?
            .text2();
        let when = element
            .select_one("div.newstc>div.newsrecent")?
            .hltv_parse_err("Cannot find 'when' for news brief")?
            .text2();
        let comments_num = element
            .select_one("div.newstc>div.newsrecent+div")?
            .hltv_parse_err("Cannot find comments for news brief")?
            .text2();

        Ok(ArticleBrief {
            path,
            name,
            when,
            comments_num,
        })
    }
}

#[derive(Debug)]
pub struct MainPageArticleBriefs {
    pub today: Vec<ArticleBrief>,
    pub yesterday: Vec<ArticleBrief>,
    pub older: Vec<ArticleBrief>,
}

impl MainPageArticleBriefs {
    // TODO: doc
    // TODO: unify code with function `archived_article_briefs_from_html`
    pub fn from_html(document: &Html) -> Result<Self> {
        let selector = Selector::parse("h2.newsheader+div.standard-box")?;
        let a_selector = Selector::parse("a")?;

        let elements = document.select(&selector).collect::<Vec<_>>();

        let parse_list = |element: ElementRef| {
            element
                .select(&a_selector)
                .map(ArticleBrief::from_element_ref)
                .collect::<Result<Vec<ArticleBrief>>>()
        };

        let briefs_blocks = elements
            .into_iter()
            .map(parse_list)
            .collect::<Result<Vec<_>>>()?;
        let mut briefs_blocks_iter = briefs_blocks.into_iter();
        let today = briefs_blocks_iter
            .next()
            .hltv_parse_err("Not todays news")?;
        let yesterday = briefs_blocks_iter
            .next()
            .hltv_parse_err("Not yesterdays news")?;
        let older = briefs_blocks_iter.flatten().collect();
        let briefs = Self {
            today,
            yesterday,
            older,
        };

        Ok(briefs)
    }
}

pub fn archived_article_briefs_from_html(document: &Html) -> Result<Vec<ArticleBrief>> {
    let selector = Selector::parse("h2.newsheader+div.standard-box")?;
    let a_selector = Selector::parse("a")?;

    let elements = document.select(&selector).collect::<Vec<_>>();

    let parse_list = |element: ElementRef| {
        element
            .select(&a_selector)
            .map(ArticleBrief::from_element_ref)
            .collect::<Result<Vec<ArticleBrief>>>()
    };

    let briefs_blocks = elements
        .into_iter()
        .map(parse_list)
        .collect::<Result<Vec<_>>>()?;
    let mut briefs_blocks_iter = briefs_blocks.into_iter();
    let briefs = briefs_blocks_iter
        .next()
        .hltv_parse_err("Not archived news")?;
    Ok(briefs)
}

#[cfg(test)]
mod article_brief_tests {
    use super::*;

    use scraper::{Html, Selector};

    fn get_elem<'a>(html: &'a Html, css_selector: &'_ str) -> ElementRef<'a> {
        html.select(&Selector::parse(css_selector).unwrap())
            .next()
            .unwrap()
    }

    #[test]
    fn parse() {
        let html = Html::parse_fragment(
            r#"
<a href="/news/30594/flashpoint-2-fantasy-game-live-with-prizes" class="newsline article" data-link-tracking-page="Frontpage" data-link-tracking-column="[Main content]" data-link-tracking-destination="Click on Standard inline news post [button]"><img alt="Europe" src="/img/static/flags/30x20/EU.gif" class="newsflag flag" title="Europe">
  <div class="newstext">Flashpoint 2 Fantasy game live with prizes</div>
  <div class="newstc">
    <div class="newsrecent">5 hours ago</div>
    <div>29 comments</div>
  </div>
</a>
"#,
        );

        assert_eq!(
            ArticleBrief::from_element_ref(get_elem(&html, "a")),
            Ok(ArticleBrief {
                name: "Flashpoint 2 Fantasy game live with prizes".into(),
                path: "/news/30594/flashpoint-2-fantasy-game-live-with-prizes".into(),
                when: "5 hours ago".into(),
                comments_num: "29 comments".into(),
            })
        )
    }

    #[test]
    fn single_parse_no_path() {
        let html = Html::parse_fragment(
            r#"
<a class="newsline article" data-link-tracking-page="Frontpage" data-link-tracking-column="[Main content]" data-link-tracking-destination="Click on Standard inline news post [button]"><img alt="Europe" src="/img/static/flags/30x20/EU.gif" class="newsflag flag" title="Europe">
  <div class="newstext">Flashpoint 2 Fantasy game live with prizes</div>
  <div class="newstc">
    <div class="newsrecent">5 hours ago</div>
    <div>29 comments</div>
  </div>
</a>
"#,
        );

        assert!(ArticleBrief::from_element_ref(get_elem(&html, "a")).is_err())
    }

    #[test]
    fn single_parse_no_name() {
        let html = Html::parse_fragment(
            r#"
<a href="/news/30594/flashpoint-2-fantasy-game-live-with-prizes" class="newsline article" data-link-tracking-page="Frontpage" data-link-tracking-column="[Main content]" data-link-tracking-destination="Click on Standard inline news post [button]"><img alt="Europe" src="/img/static/flags/30x20/EU.gif" class="newsflag flag" title="Europe">
  <div>Flashpoint 2 Fantasy game live with prizes</div>
  <div class="newstc">
    <div class="newsrecent">5 hours ago</div>
    <div>29 comments</div>
  </div>
</a>
"#,
        );

        assert!(ArticleBrief::from_element_ref(get_elem(&html, "a")).is_err())
    }

    #[test]
    fn single_parse_no_when() {
        let html = Html::parse_fragment(
            r#"
<a href="/news/30594/flashpoint-2-fantasy-game-live-with-prizes" class="newsline article" data-link-tracking-page="Frontpage" data-link-tracking-column="[Main content]" data-link-tracking-destination="Click on Standard inline news post [button]"><img alt="Europe" src="/img/static/flags/30x20/EU.gif" class="newsflag flag" title="Europe">
  <div class="newstext">Flashpoint 2 Fantasy game live with prizes</div>
  <div class="newstc">
    <div class="">5 hours ago</div>
    <div>29 comments</div>
  </div>
</a>
"#,
        );

        assert!(ArticleBrief::from_element_ref(get_elem(&html, "a")).is_err())
    }

    #[test]
    fn single_parse_no_comments_num() {
        let html = Html::parse_fragment(
            r#"
<a href="/news/30594/flashpoint-2-fantasy-game-live-with-prizes" class="newsline article" data-link-tracking-page="Frontpage" data-link-tracking-column="[Main content]" data-link-tracking-destination="Click on Standard inline news post [button]"><img alt="Europe" src="/img/static/flags/30x20/EU.gif" class="newsflag flag" title="Europe">
  <div class="newstext">Flashpoint 2 Fantasy game live with prizes</div>
  <div class="newstc">
    <div class="newsrecent">5 hours ago</div>
  </div>
</a>
"#,
        );

        assert!(ArticleBrief::from_element_ref(get_elem(&html, "a")).is_err())
    }

    #[test]
    fn main_page_parse() {
        let html = Html::parse_fragment(
            r#"
<div class="index">
  <h2 class="newsheader">Today's news</h2>
  <div class="standard-box standard-list">
    <a href="/news/30594/flashpoint-2-fantasy-game-live-with-prizes" class="newsline article" data-link-tracking-page="Frontpage" data-link-tracking-column="[Main content]" data-link-tracking-destination="Click on Standard inline news post [button]"><img alt="Europe" src="/img/static/flags/30x20/EU.gif" class="newsflag flag" title="Europe">
      <div class="newstext">Flashpoint 2 Fantasy game live with prizes</div>
      <div class="newstc">
        <div class="newsrecent">6 hours ago</div>
        <div>29 comments</div>
      </div>
    </a>
    <a href="/news/30590/video-flamez-vs-gambit" class="newsline article" data-link-tracking-page="Frontpage" data-link-tracking-column="[Main content]" data-link-tracking-destination="Click on Standard inline news post [button]"><img alt="Israel" src="/img/static/flags/30x20/IL.gif" class="newsflag flag" title="Israel">
      <div class="newstext">Video: flameZ vs. Gambit</div>
      <div class="newstc">
        <div class="newsrecent">18 hours ago</div>
        <div>44 comments</div>
      </div>
    </a>
  </div>

  <div> Some element </div>

  <h2 class="newsheader">Yesterday's news</h2>
  <div class="standard-box standard-list">
    <a href="/news/30589/european-development-championship-series-announced-with-150000" class="newsline article" data-link-tracking-page="Frontpage" data-link-tracking-column="[Main content]" data-link-tracking-destination="Click on Standard inline news post [button]"><img alt="Europe" src="/img/static/flags/30x20/EU.gif" class="newsflag flag" title="Europe">
      <div class="newstext">European Development Championship series announced with $150,000</div>
      <div class="newstc">
        <div class="newsrecent">a day ago</div>
        <div>55 comments</div>
      </div>
    </a>
    <a href="/news/30588/thorin-announces-consulting-partnership-with-guild" class="newsline article" data-link-tracking-page="Frontpage" data-link-tracking-column="[Main content]" data-link-tracking-destination="Click on Standard inline news post [button]"><img alt="United Kingdom" src="/img/static/flags/30x20/GB.gif" class="newsflag flag" title="United Kingdom">
      <div class="newstext">Thorin announces consulting partnership with Guild</div>
      <div class="newstc">
        <div class="newsrecent">a day ago</div>
        <div>267 comments</div>
      </div>
    </a>
    <a href="/news/30586/video-top-10-highlights-of-october" class="newsline article" data-link-tracking-page="Frontpage" data-link-tracking-column="[Main content]" data-link-tracking-destination="Click on Standard inline news post [button]"><img alt="Other" src="/img/static/flags/30x20/WORLD.gif" class="newsflag flag" title="Other">
      <div class="newstext">Video: Top 10 highlights of October</div>
      <div class="newstc">
        <div class="newsrecent">a day ago</div>
        <div>96 comments</div>
      </div>
    </a>
  </div>

  <div> Some element </div>

  <div class="old-news-con">
    <h2 class="newsheader">Previous news</h2>
    <div class="standard-box standard-list">
      <a href="/news/30587/godsent-target-emi-as-potential-replacement-for-krystal" class="newsline article" data-link-tracking-page="Frontpage" data-link-tracking-column="[Main content]" data-link-tracking-destination="Click on Standard inline news post [button]"><img alt="Europe" src="/img/static/flags/30x20/EU.gif" class="newsflag flag" title="Europe">
        <div class="newstext">GODSENT target emi as potential replacement for kRYSTAL</div>
        <div class="newstc">
          <div class="newsrecent">2 days ago</div>
          <div>346 comments</div>
        </div>
      </a>
      <a href="/news/30585/ence-sign-doto-on-two-year-deal-saw-joins-as-head-coach" class="newsline article" data-link-tracking-page="Frontpage" data-link-tracking-column="[Main content]" data-link-tracking-destination="Click on Standard inline news post [button]"><img alt="Finland" src="/img/static/flags/30x20/FI.gif" class="newsflag flag" title="Finland">
        <div class="newstext">ENCE sign doto on two-year deal; sAw joins as head coach</div>
        <div class="newstc">
          <div class="newsrecent">2 days ago</div>
          <div>248 comments</div>
        </div>
      </a>
    </div>
  </div>
  <a href="/news/archive/2020/november" class="button-more">More news</a><br>
</div>
"#,
        );

        let parsed_main_page_article_briefs = MainPageArticleBriefs::from_html(&html).unwrap();
        assert_eq!(parsed_main_page_article_briefs.today.len(), 2);
        assert_eq!(parsed_main_page_article_briefs.yesterday.len(), 3);
        assert_eq!(parsed_main_page_article_briefs.older.len(), 2);
    }

    #[test]
    fn main_page_parse_err() {
        let html = Html::parse_fragment(
            r#"
<div class="index">
  <h2>Today's news</h2>
  <div class="standard-box standard-list">
    <a href="/news/30594/flashpoint-2-fantasy-game-live-with-prizes" class="newsline article" data-link-tracking-page="Frontpage" data-link-tracking-column="[Main content]" data-link-tracking-destination="Click on Standard inline news post [button]"><img alt="Europe" src="/img/static/flags/30x20/EU.gif" class="newsflag flag" title="Europe">
      <div class="newstext">Flashpoint 2 Fantasy game live with prizes</div>
      <div class="newstc">
        <div class="newsrecent">6 hours ago</div>
        <div>29 comments</div>
      </div>
    </a>
    <a href="/news/30590/video-flamez-vs-gambit" class="newsline article" data-link-tracking-page="Frontpage" data-link-tracking-column="[Main content]" data-link-tracking-destination="Click on Standard inline news post [button]"><img alt="Israel" src="/img/static/flags/30x20/IL.gif" class="newsflag flag" title="Israel">
      <div class="newstext">Video: flameZ vs. Gambit</div>
      <div class="newstc">
        <div class="newsrecent">18 hours ago</div>
        <div>44 comments</div>
      </div>
    </a>
  </div>
</div>
"#,
        );

        assert!(MainPageArticleBriefs::from_html(&html).is_err());
    }

    #[test]
    fn archive_page_parse() {
        let html = Html::parse_fragment(
            r#"
<div class="index">
  <h2 class="newsheader">News from April, 2020</h2>
  <div class="standard-box standard-list"><a href="/news/29553/mvp-pk-leave-cs" class="newsline article" data-link-tracking-page="Newsarchive" data-link-tracking-column="[Main content]" data-link-tracking-destination="Click on Archived news post [button]"><img alt="Korea" src="/img/static/flags/30x20/KR.gif" class="newsflag flag" title="Korea">
      <div class="newstext">MVP PK leave CS</div>
      <div class="newstc">
        <div class="newsrecent">2020-04-30</div>
        <div>375 comments</div>
      </div>
    </a><a href="/news/29552/ence-top-elisa-invitational-group-in-jamppis-debut-godsent-eliminated" class="newsline article" data-link-tracking-page="Newsarchive" data-link-tracking-column="[Main content]" data-link-tracking-destination="Click on Archived news post [button]"><img alt="Europe" src="/img/static/flags/30x20/EU.gif" class="newsflag flag" title="Europe">
      <div class="newstext">ENCE top Elisa Invitational group in Jamppi's debut; GODSENT eliminated</div>
      <div class="newstc">
        <div class="newsrecent">2020-04-30</div>
        <div>112 comments</div>
      </div>
    </a><a href="/news/29550/video-hallzerk-vs-ence" class="newsline article" data-link-tracking-page="Newsarchive" data-link-tracking-column="[Main content]" data-link-tracking-destination="Click on Archived news post [button]"><img alt="Norway" src="/img/static/flags/30x20/NO.gif" class="newsflag flag" title="Norway">
      <div class="newstext">Video: hallzerk vs. ENCE</div>
      <div class="newstc">
        <div class="newsrecent">2020-04-30</div>
        <div>72 comments</div>
      </div>
    </a><a href="/news/29551/og-virtuspro-spirit-headline-team-list-for-hellcase-cup-8" class="newsline article" data-link-tracking-page="Newsarchive" data-link-tracking-column="[Main content]" data-link-tracking-destination="Click on Archived news post [button]"><img alt="Europe" src="/img/static/flags/30x20/EU.gif" class="newsflag flag" title="Europe">
      <div class="newstext">OG, Virtus.pro, Spirit headline team list for Hellcase Cup 8</div>
      <div class="newstc">
        <div class="newsrecent">2020-04-30</div>
        <div>56 comments</div>
      </div>
    </a>
  </div>
</div>
"#,
        );

        let parsed_archived_article_briefs = archived_article_briefs_from_html(&html).unwrap();
        assert_eq!(parsed_archived_article_briefs.len(), 4);
    }

    #[test]
    fn archive_page_parse_err() {
        let html = Html::parse_fragment(
            r#"
<div class="index">
  <h2 class="newsheader">News from April, 2020</h2>
  <div class=""><a href="/news/29553/mvp-pk-leave-cs" class="newsline article" data-link-tracking-page="Newsarchive" data-link-tracking-column="[Main content]" data-link-tracking-destination="Click on Archived news post [button]"><img alt="Korea" src="/img/static/flags/30x20/KR.gif" class="newsflag flag" title="Korea">
      <div class="newstext">MVP PK leave CS</div>
      <div class="newstc">
        <div class="newsrecent">2020-04-30</div>
        <div>375 comments</div>
      </div>
    </a><a href="/news/29552/ence-top-elisa-invitational-group-in-jamppis-debut-godsent-eliminated" class="newsline article" data-link-tracking-page="Newsarchive" data-link-tracking-column="[Main content]" data-link-tracking-destination="Click on Archived news post [button]"><img alt="Europe" src="/img/static/flags/30x20/EU.gif" class="newsflag flag" title="Europe">
      <div class="newstext">ENCE top Elisa Invitational group in Jamppi's debut; GODSENT eliminated</div>
      <div class="newstc">
        <div class="newsrecent">2020-04-30</div>
        <div>112 comments</div>
      </div>
    </a><a href="/news/29550/video-hallzerk-vs-ence" class="newsline article" data-link-tracking-page="Newsarchive" data-link-tracking-column="[Main content]" data-link-tracking-destination="Click on Archived news post [button]"><img alt="Norway" src="/img/static/flags/30x20/NO.gif" class="newsflag flag" title="Norway">
      <div class="newstext">Video: hallzerk vs. ENCE</div>
      <div class="newstc">
        <div class="newsrecent">2020-04-30</div>
        <div>72 comments</div>
      </div>
    </a><a href="/news/29551/og-virtuspro-spirit-headline-team-list-for-hellcase-cup-8" class="newsline article" data-link-tracking-page="Newsarchive" data-link-tracking-column="[Main content]" data-link-tracking-destination="Click on Archived news post [button]"><img alt="Europe" src="/img/static/flags/30x20/EU.gif" class="newsflag flag" title="Europe">
      <div class="newstext">OG, Virtus.pro, Spirit headline team list for Hellcase Cup 8</div>
      <div class="newstc">
        <div class="newsrecent">2020-04-30</div>
        <div>56 comments</div>
      </div>
    </a>
  </div>
</div>
"#,
        );

        assert!(archived_article_briefs_from_html(&html).is_err());
    }
}
