use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use pyo3::{
    create_exception,
    exceptions::{PyException, PyIOError},
    prelude::*,
    types::{PyDict, PyType},
};
use serde_json::Value;
use thiserror::Error;
use tokio_postgres::NoTls;

create_exception!(pgstars, PgstacError, PyException);

#[pyclass]
struct Client(Pool<PostgresConnectionManager<NoTls>>);

#[pymethods]
impl Client {
    #[classmethod]
    fn open<'a>(
        _: Bound<'_, PyType>,
        py: Python<'a>,
        params: String,
    ) -> PyResult<Bound<'a, PyAny>> {
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let connection_manager = PostgresConnectionManager::new_from_stringlike(params, NoTls)
                .map_err(Error::from)?;
            let pool = Pool::builder()
                .build(connection_manager)
                .await
                .map_err(Error::from)?;
            Ok(Client(pool))
        })
    }

    fn _execute_query<'a>(&mut self, py: Python<'a>, query: String) -> PyResult<Bound<'a, PyAny>> {
        let pool = self.0.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let client = pool.get().await.map_err(Error::from)?;
            client.execute(&query, &[]).await.map_err(Error::from)?;
            Ok(())
        })
    }

    fn get_version<'a>(&mut self, py: Python<'a>) -> PyResult<Bound<'a, PyAny>> {
        let pool = self.0.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let client = pool.get().await.map_err(Error::from)?;
            pgstac::Client::new(&*client)
                .version()
                .await
                .map_err(|err| Error::from(err).into())
        })
    }

    fn get_collection<'a>(&mut self, py: Python<'a>, id: String) -> PyResult<Bound<'a, PyAny>> {
        let pool = self.0.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let client = pool.get().await.map_err(Error::from)?;
            let value = pgstac::Client::new(&*client)
                .opt::<Value>("get_collection", &[&id])
                .await
                .map_err(Error::from)?;
            if let Some(value) = value {
                Ok(Some(Json(value)))
            } else {
                Ok(None)
            }
        })
    }

    fn create_collection<'a>(
        &mut self,
        py: Python<'a>,
        collection: Bound<'_, PyDict>,
    ) -> PyResult<Bound<'a, PyAny>> {
        let value: Value = pythonize::depythonize(&collection)?;
        let pool = self.0.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let client = pool.get().await.map_err(Error::from)?;
            pgstac::Client::new(&*client)
                .void("create_collection", &[&value])
                .await
                .map_err(Error::from)?;
            Ok(())
        })
    }
}

struct Json(serde_json::Value);

impl IntoPy<PyObject> for Json {
    fn into_py(self, py: Python<'_>) -> PyObject {
        pythonize::pythonize(py, &self.0).unwrap().into()
    }
}

#[derive(Debug, Error)]
enum Error {
    #[error(transparent)]
    Pgstac(#[from] pgstac::Error),
    #[error(transparent)]
    RunTokioPostgres(#[from] bb8::RunError<tokio_postgres::Error>),
    #[error(transparent)]
    TokioPostgres(#[from] tokio_postgres::Error),
}

impl From<Error> for PyErr {
    fn from(err: Error) -> Self {
        match err {
            Error::Pgstac(_) => PgstacError::new_err(format!("pgstac error: {}", err)),
            Error::TokioPostgres(_) | Error::RunTokioPostgres(_) => {
                PyIOError::new_err(format!("tokio postgres error: {}", err))
            }
        }
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn pgstacrs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Client>()?;
    Ok(())
}
