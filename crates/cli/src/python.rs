use crate::Args;
use clap::Parser;
use pyo3::{
    prelude::{PyModule, PyModuleMethods},
    pyfunction, pymodule,
    types::PyAnyMethods,
    wrap_pyfunction, Bound, PyResult, Python,
};

#[pyfunction]
fn main(py: Python<'_>) -> PyResult<i64> {
    let signal = py.import("signal")?;
    let _ = signal
        .getattr("signal")?
        .call1((signal.getattr("SIGINT")?, signal.getattr("SIG_DFL")?))?;
    let args = Args::parse_from(std::env::args_os().skip(1));
    tracing_subscriber::fmt()
        .with_max_level(args.log_level())
        .init();
    std::process::exit(
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                // We skip one because the first argument is going to be the python interpreter.
                match args.run().await {
                    Ok(()) => 0,
                    Err(err) => {
                        eprintln!("ERROR: {}", err);
                        err.code()
                    }
                }
            }),
    )
}

#[pymodule]
fn stacrs_cli(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(main, m)?)?;
    Ok(())
}
