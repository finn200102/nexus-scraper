use pyo3::prelude::*;
use nexus_core::sites::{get_site, Site as Site};
use nexus_core::detect_site_from_url;
use pyo3_asyncio::tokio::future_into_py;
use std::sync::Arc;
use reqwest::Client;

#[pyclass]
#[derive(Clone)]
struct PyChapter {
    #[pyo3(get)]
    site: String,
    #[pyo3(get)]
    title: String,
    #[pyo3(get)]
    text: String,
    #[pyo3(get)]
    chapter_number: u32,
    #[pyo3(get)]
    chapter_id: u64,
    #[pyo3(get)]
    url: String,
}

#[pyclass]
#[derive(Clone)]
struct PyStory {
    #[pyo3(get)]
    site: String,
    #[pyo3(get)]
    story_name: String,
    #[pyo3(get)]
    story_id: u64,
    #[pyo3(get)]
    author_name: String,
    #[pyo3(get)]
    author_id: u64,
    #[pyo3(get)]
    chapters: Vec<PyChapter>,
    #[pyo3(get)]
    description: String,
    #[pyo3(get)]
    img_url: String,
    #[pyo3(get)]
    word_count: u64,
    #[pyo3(get)]
    reviews: u64,
    #[pyo3(get)]
    favorites: u64,
    #[pyo3(get)]
    follows: u64,
    #[pyo3(get)]
    publish_date: String,
    #[pyo3(get)]
    updated_date: String,
    #[pyo3(get)]
    status: String,
    #[pyo3(get)]
    views: u64,
    #[pyo3(get)]
    rating: f64,
    #[pyo3(get)]
    chapter_count: u64,
    #[pyo3(get)]
    url: String,
    #[pyo3(get)]
    story_not_found: bool,
}

#[pyclass]
struct PyStories {
    #[pyo3(get)]
    stories: Vec<PyStory>,
}


#[pyclass]
struct PySite {
    site: Arc<dyn Site + Send + Sync>,
    client: Client,
}

#[pymethods]
impl PySite {
    #[new]
    fn new(name: &str) -> PyResult<Self> {
        let site = get_site(name)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(Self {
            site,
            client: Client::new(),
        })
    }

    fn fetch_story_from_url<'py>(
        &'py self,
        py: Python<'py>,
        url: String,
    ) -> PyResult<&'py PyAny> {
        let site = self.site.clone();
        let client = self.client.clone();
        future_into_py(py, async move {
            let story_result = site
                .get_story_data_from_url(&url, &client)
                .await;

            Python::with_gil(|py| {
                match story_result {
                    Ok(story) => Ok(Py::new(
                        py,
                        PyStory {
                            site: story.site,
                            story_name: story.story_name.unwrap_or_default(),
                            story_id: story.story_id.unwrap_or_default(),
                            author_name: story.author_name.unwrap_or_default(),
                            author_id: story.author_id.unwrap_or_default(),
                            description: story.description.unwrap_or_default(),
                            img_url: story.img_url.unwrap_or_default(),
                            word_count: story.word_count.unwrap_or_default(),
                            reviews: story.reviews.unwrap_or_default(),
                            favorites: story.favorites.unwrap_or_default(),
                            follows: story.follows.unwrap_or_default(),
                            publish_date: story.publish_date.unwrap_or_default(),
                            updated_date: story.updated_date.unwrap_or_default(),
                            status: story.status.unwrap_or_default(),
                            views: story.views.unwrap_or_default(),
                            rating: story.rating.unwrap_or_default(),
                            chapter_count: story.chapter_count.unwrap_or_default(),
                            url: story.url.unwrap_or_default(),
                            chapters: story.chapters
                                .into_iter()
                                .map(|chap| PyChapter {
                                    site: chap.site,
                                    title: chap.title.unwrap_or_default(),
                                    text: chap.text.unwrap_or_default(),
                                    chapter_number: chap.chapter_number.unwrap_or(0),
                                    chapter_id: chap.chapter_id.unwrap_or(0),
                                    url: chap.url.unwrap_or_default(),
                                })
                                .collect(),
                            story_not_found: false,
                        }
                        )?
                        .into_py(py)),
                    Err(e) => {
                        if let Some(nexus_core::error::CoreError::StoryNotFound(_)) = e.downcast_ref() {
                            let split: Vec<_> = url.split('/').collect();
                            let story_id = split.get(4).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
                            Ok(Py::new(
                                py,
                                PyStory {
                                    site: "fanfiction".to_string(),
                                    story_name: "Story Not Found".to_string(),
                                    story_id,
                                    author_name: "".to_string(),
                                    author_id: 0,
                                    description: "".to_string(),
                                    img_url: "".to_string(),
                                    word_count: 0,
                                    reviews: 0,
                                    favorites: 0,
                                    follows: 0,
                                    publish_date: "".to_string(),
                                    updated_date: "".to_string(),
                                    status: "".to_string(),
                                    views: 0,
                                    rating: 0.0,
                                    chapter_count: 0,
                                    url: url.clone(),
                                    chapters: vec![],
                                    story_not_found: true,
                                }
                            )?.into_py(py))
                        } else {
                            Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
                        }
                    }
                }
            })
        })
    }

    fn fetch_chapter<'py>(
        &'py self,
        py: Python<'py>,
        story_id: u64,
        chapter_id: u64,
        chapter_number: u32,
    ) -> PyResult<&'py PyAny> {
        let site = self.site.clone();
        let client = self.client.clone();
        future_into_py(py, async move {
            let chapter = site
                .fetch_chapter(story_id, chapter_id, chapter_number, &client)
                .await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

            Python::with_gil(|py| {
                Ok(Py::new(
                    py,
                    PyChapter {
                        site: chapter.site,
                        title: chapter.title.unwrap_or_default(),
                        text: chapter.text.unwrap_or_default(),
                        chapter_number: chapter.chapter_number.unwrap_or(0),
                        chapter_id: chapter.chapter_id.unwrap_or(0),
                        url: chapter.url.unwrap_or_default(),
                    }
                )?
                .into_py(py))
            })
        })
    }

    fn fetch_stories_by_series<'py>(
        &'py self,
        py: Python<'py>,
        medium_name: String,
        series_name: String,
        sortby_id: u32,
        rating_id: u32,
        word_count: u32,
        time_range: u32,
        num_pages: u32,
    ) -> PyResult<&'py PyAny> {
        let site = self.site.clone();
        let client = self.client.clone();
        future_into_py(py, async move {
            let stories = site
                .fetch_stories_by_series(
                    medium_name,
                    &series_name,
                    sortby_id,
                    rating_id,
                    word_count,
                    time_range,
                    num_pages,
                    &client,
                )
                .await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

            Python::with_gil(|py| {
                Ok(Py::new(
                    py,
                    PyStories {
                        stories: stories.stories
                            .into_iter()
                            .map(|s| PyStory {
                                site: s.site,
                                story_name: s.story_name.unwrap_or_default(),
                                story_id: s.story_id.unwrap_or_default(),
                                author_name: s.author_name.unwrap_or_default(),
                                author_id: s.author_id.unwrap_or_default(),
                                description: s.description.unwrap_or_default(),
                                img_url: s.img_url.unwrap_or_default(),
                                word_count: s.word_count.unwrap_or_default(),
                                reviews: s.reviews.unwrap_or_default(),
                                favorites: s.favorites.unwrap_or_default(),
                                follows: s.follows.unwrap_or_default(),
                                publish_date: s.publish_date.unwrap_or_default(),
                                updated_date: s.updated_date.unwrap_or_default(),
                                status: s.status.unwrap_or_default(),
                                views: s.views.unwrap_or_default(),
                                rating: s.rating.unwrap_or_default(),
                                chapter_count: s.chapter_count.unwrap_or_default(),
                                url: s.url.unwrap_or_default(),
                                chapters: vec![],
                                story_not_found: false,
                            })
                            .collect(),
                    },
                )?
                .into_py(py))
            })
        })
    }
}

#[pyfunction]
fn fetch_story<'py>(py: Python<'py>, url: String) -> PyResult<&'py PyAny> {
    future_into_py(py, async move {
        let site_name = detect_site_from_url(&url).map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        let site = get_site(site_name).map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        let client = Client::new();
        let story_result = site
            .get_story_data_from_url(&url, &client)
            .await;

        Python::with_gil(|py| {
            match story_result {
                Ok(story) => Ok(Py::new(
                        py,
                        PyStory {
                            site: story.site,
                            story_name: story.story_name.unwrap_or_default(),
                            story_id: story.story_id.unwrap_or_default(),
                            author_name: story.author_name.unwrap_or_default(),
                            author_id: story.author_id.unwrap_or_default(),
                            description: story.description.unwrap_or_default(),
                            img_url: story.img_url.unwrap_or_default(),
                            word_count: story.word_count.unwrap_or_default(),
                            reviews: story.reviews.unwrap_or_default(),
                            favorites: story.favorites.unwrap_or_default(),
                            follows: story.follows.unwrap_or_default(),
                            publish_date: story.publish_date.unwrap_or_default(),
                            updated_date: story.updated_date.unwrap_or_default(),
                            status: story.status.unwrap_or_default(),
                            views: story.views.unwrap_or_default(),
                            rating: story.rating.unwrap_or_default(),
                            chapter_count: story.chapter_count.unwrap_or_default(),
                            url: story.url.unwrap_or_default(),
                            chapters: story.chapters
                                .into_iter()
                                .map(|chap| PyChapter {
                                    site: chap.site,
                                    title: chap.title.unwrap_or_default(),
                                    text: chap.text.unwrap_or_default(),
                                    chapter_number: chap.chapter_number.unwrap_or(0),
                                    chapter_id: chap.chapter_id.unwrap_or(0),
                                    url: chap.url.unwrap_or_default(),
                                })
                                .collect(),
                            story_not_found: false,
                        }
                        )?
                        .into_py(py)),
                Err(e) => {
                    if let Some(nexus_core::error::CoreError::StoryNotFound(_)) = e.downcast_ref() {
                        let split: Vec<_> = url.split('/').collect();
                        let story_id = split.get(4).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
                        Ok(Py::new(
                            py,
                            PyStory {
                                site: "fanfiction".to_string(),
                                story_name: "Story Not Found".to_string(),
                                story_id,
                                author_name: "".to_string(),
                                author_id: 0,
                                description: "".to_string(),
                                img_url: "".to_string(),
                                word_count: 0,
                                reviews: 0,
                                favorites: 0,
                                follows: 0,
                                publish_date: "".to_string(),
                                updated_date: "".to_string(),
                                status: "".to_string(),
                                views: 0,
                                rating: 0.0,
                                chapter_count: 0,
                                url: url.clone(),
                                chapters: vec![],
                                story_not_found: true,
                            }
                        )?.into_py(py))
                    } else {
                        Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
                    }
                }
            }
        })
    })
}

#[pymodule]
fn pybindings(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PySite>()?;
    m.add_class::<PyStories>()?;
    m.add_function(wrap_pyfunction!(fetch_story, m)?)?;
    Ok(())
}
