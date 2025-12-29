use anyhow::Result;
use serde::{Deserialize, Serialize};

const PISTON_URL: &str = "https://launchercontent.mojang.com/news.json";
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct News {
    pub version: u8,
    pub entries: Vec<NewsReport>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NewsReport {
    pub id: String,
    pub title: String,
    pub category: Category,
    pub date: String,
    pub text: String,
    #[serde(rename = "playPageImage")]
    pub play_page_image: NewsImage,
    #[serde(rename = "newsPageImage")]
    pub news_page_image: NewsImage,
    #[serde(rename = "readMoreLink")]
    pub read_more_link: String,
    #[serde(rename = "newsType")]
    pub news_type: Vec<String>,
    #[serde(rename = "cardBorder")]
    pub card_border: Option<bool>,
    pub tag: Option<String>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NewsImage {
    pub title: String,
    pub url: String,
    pub dimensions: Option<ImageDimensions>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum Category {
    #[serde(rename = "Minecraft Legends")]
    MinecraftLegends,
    #[serde(rename = "Minecraft for Windows")]
    MinecraftForWindows,
    #[serde(rename = "Minecraft: Java Edition")]
    MinecraftJavaEdition,
    #[serde(rename = "Minecraft Dungeons")]
    MinecraftDungeons,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageDimensions {
    pub width: u16,
    pub height: u16,
}

impl News {
    pub async fn fetch() -> Result<News> {
        let response = reqwest::get(PISTON_URL).await?;
        Ok(response.json().await?)
    }
    pub async fn java_edition(&self) -> Vec<NewsReport> {
        self.get_news_by_category(Category::MinecraftJavaEdition)
    }

    pub async fn minecraft_windows(&self) -> Vec<NewsReport> {
        self.get_news_by_category(Category::MinecraftForWindows)
    }

    pub async fn dungeons(&self) -> Vec<NewsReport> {
        self.get_news_by_category(Category::MinecraftDungeons)
    }

    pub async fn legends(&self) -> Vec<NewsReport> {
        self.get_news_by_category(Category::MinecraftLegends)
    }

    pub fn get_news_by_category(&self, category: Category) -> Vec<NewsReport> {
        self.entries.iter().filter(|entry| entry.category == category).cloned().collect()
    }
}

#[cfg(test)]
mod test {
    use crate::news::News;
    use crate::setup_logging;
    #[tokio::test]
    async fn get_news() {
        #[cfg(feature = "log")]
        setup_logging();
        let news = News::fetch().await.unwrap();
        assert!(!news.entries.is_empty())
    }
}
