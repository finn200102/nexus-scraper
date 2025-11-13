use pyo4::prelude::*;
use nexus_core::sites::{get_site, Site as Site};
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
}

#[pyclass]
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
            let story = site
                .get_story_data_from_url(&url, &client)
                .await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
            
            Python::with_gil(|py| {
                Ok(Py::new(
                        py,
                        PyStory {
                            site: story.site,
                            story_name: story.story_name.unwrap_or_default(),
                            story_id: story.story_id.unwrap_or_default(),
                            author_name: story.author_name.unwrap_or_default(),
                            author_id: story.author_id.unwrap_or_default(),
                            description: story.description.unwrap_or_default(),
                            chapters: story.chapters
                                .into_iter()
                                .map(|chap| PyChapter {
                                    site: chap.site,
                                    title: chap.title.unwrap_or_default(),
                                    text: chap.text.unwrap_or_default(),
                                    chapter_number: chap.chapter_number.unwrap_or(0),
                                    chapter_id: chap.chapter_id.unwrap_or(0),
                                })
                                .collect(),
                        }
                                )?
                                .into_py(py))
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
                        site: chapter.site,  // assuming this is always present
                        title: chapter.title.unwrap_or_default(),
                        text: chapter.text.unwrap_or_default(),
                        chapter_number: chapter.chapter_number.unwrap_or(0),
                        chapter_id: chapter.chapter_id.unwrap_or(0),
                    }

          
                )?
                .into_py(py))
            })
        })
    }
}

use pyo3::prelude::*;

#[pymodule]
fn pybindings(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PySite>()?;
    // Add more classes or functions as needed
    Ok(())
}


