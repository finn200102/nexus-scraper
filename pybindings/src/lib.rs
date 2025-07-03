use pyo3::prelude::*;
use nexus_core::sites::{get_site, Site as CoreSite};

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
    site: CoreSite,
}

#[pymethods]
impl PySite {
    #[new]
    fn new(name: &str) -> Self {
        Self {
            site: get_site(name),
        }
    }

    fn fetch_chapter(&self, story_id: &str, chapter_id: u64, chapter_number: u32) -> PyChapter {
        let chapter = self.site.fetch(story_id, chapter_id, chapter_number);
        PyChapter {
            site: story.site,
            title: story.title,
            text: story.text,
            chapter_number: story.chapter_number,
            chapter_id: story.chapter_id,
        }
    }
}

#[pymodule]
fn pybindings(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PySite>()?;
    m.add_class::<PyChapter>()?;
    Ok(())
}

