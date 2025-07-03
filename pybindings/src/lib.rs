use pyo3::prelude::*;
use nexus_core::sites::{get_site, Site as Site};
use pyo3_asyncio::tokio::future_into_py;
use std::sync::Arc;
use reqwest::Client;

#[pyclass]
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
            site: Arc::from(site),
            client: Client::new(),
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
                        title: chapter.title,
                        text: chapter.text,
                        chapter_number: chapter.chapter_number,
                        chapter_id: chapter.chapter_id,
                    },
                )?
                .into_py(py))
            })
        })
    }
}

