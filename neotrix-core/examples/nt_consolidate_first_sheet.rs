fn main() {
    let src = std::path::PathBuf::from(std::env::args().nth(1).expect("src dir"));
    let out = std::path::PathBuf::from(std::env::args().nth(2).expect("output"));
    match neotrix::neotrix::consolidate_tables_first_sheet(&src, &out) {
        Ok(rep) => {
            println!(
                "files={} rows={} usd={} failed={:?}",
                rep.files_processed, rep.total_rows, rep.usd_rows, rep.files_failed
            );
            if !rep.validation_warnings.is_empty() {
                println!("warnings={}", rep.validation_warnings.len());
            }
        }
        Err(e) => println!("error={e}"),
    }
}