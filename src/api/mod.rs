pub use article::*;

use scraper::{ElementRef, Html, Selector};

use crate::{AttoHttpcImpl, Error, HttpsClient, NoneErrorExt, Result, HLTV_URL};

mod article;

/// Extension trait for `scrapper::ElementRef`.
trait ElementRefExt {
    /// Get text inside HTML element.
    fn text2(&self) -> String;
    /// Select first HTML element inside current by CSS selector.
    fn select_one(&self, selector: &str) -> Result<Option<Self>>
    where
        Self: Sized;
}

impl<'a> ElementRefExt for ElementRef<'a> {
    fn text2(&self) -> String {
        self.text().collect::<Vec<_>>().join("")
    }

    fn select_one(&self, selector: &str) -> Result<Option<ElementRef<'a>>> {
        let selector_parsed = Selector::parse(selector)?;
        Ok(self.select(&selector_parsed).next())
    }
}

pub enum Month {
    January,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,
}

impl Month {
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::January => "january",
            Self::February => "february",
            Self::March => "march",
            Self::April => "april",
            Self::May => "may",
            Self::June => "june",
            Self::July => "july",
            Self::August => "august",
            Self::September => "september",
            Self::October => "october",
            Self::November => "november",
            Self::December => "december",
        }
    }
}

// HLTV API
pub struct HltvApi {
    https_client: Box<dyn HttpsClient>,
    hltv_root_url: String,
}

impl HltvApi {
    /// Build new instance of `HltvApi` with provided HTTPS client and HLTV URL.
    pub fn new<T: HttpsClient + 'static, U: Into<String>>(client: T, hltv_root_url: U) -> Self {
        Self {
            https_client: Box::new(client),
            hltv_root_url: hltv_root_url.into(),
        }
    }

    /// Build new instance of `HltvApi` with provided HTTPS client and default HLTV URL.
    pub fn with_default_path<T: HttpsClient + 'static>(client: T) -> Self {
        Self {
            https_client: Box::new(client),
            hltv_root_url: HLTV_URL.into(),
        }
    }

    fn get_page(&self, path: &str) -> Result<String> {
        self.https_client
            .get(&format!("{}{}", self.hltv_root_url, path))
            .map_err(|err| Error::HttpsClient(err))
    }

    /// Get news briefs from main page (ie latest news).
    pub fn latest_news_briefs(&self) -> Result<MainPageArticleBriefs> {
        let document = Html::parse_document(&self.get_page("/")?);
        MainPageArticleBriefs::from_html(&document)
    }

    /// Get news briefs from archive.
    pub fn archived_news_briefs(&self, year: u16, month: Month) -> Result<Vec<ArticleBrief>> {
        let path = format!("/news/archive/{}/{}", year, month.to_str());
        let document = Html::parse_document(&self.get_page(&path)?);
        archived_article_briefs_from_html(&document)
    }
}

/// Build new instance of `HltvApi` with `attohttpc` client and default HLTV URL.
#[cfg(feature = "attohttpc_client")]
impl Default for HltvApi {
    fn default() -> Self {
        Self {
            https_client: Box::new(AttoHttpcImpl {}),
            hltv_root_url: HLTV_URL.into(),
        }
    }
}

// TODO: throttle requests to HLTV, otherwise get banned by Cloudflare
#[cfg(all(test, feature = "attohttpc_client"))]
mod tests {
    use super::*;

    #[test]
    fn get_hltv_root() {
        assert!(HltvApi::default().get_page("/").is_ok());
    }

    #[test]
    fn get_hltv_404() {
        assert!(HltvApi::default().get_page("/unknown_resource").is_err());
    }

    #[test]
    fn latest_news_briefs() {
        assert!(HltvApi::default().latest_news_briefs().is_ok());
    }

    #[test]
    fn archived_news_briefs() {
        let months = vec![
            Month::January,
            Month::February,
            Month::March,
            Month::April,
            Month::May,
            Month::June,
            Month::July,
            Month::August,
            Month::September,
            Month::October,
            Month::November,
            Month::December,
        ];

        for month in months {
            assert!(HltvApi::default().archived_news_briefs(2019, month).is_ok());
        }
    }
}
