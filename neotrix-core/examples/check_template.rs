use std::path::Path;
use office_oxide::Document;

fn main() {
    let path = Path::new("/Users/neo/Downloads/price/data/格式.xlsx");
    let doc = Document::open(path).expect("Failed to open template");
    
    if let Some(xlsx) = doc.as_xlsx() {
        println!("Template sheets: {:?}", xlsx.worksheets.iter().map(|ws| &ws.name).collect::<Vec<_>>());
        
        for (i, ws) in xlsx.worksheets.iter().enumerate() {
            println!("\nSheet {}: {}", i+1, ws.name);
            println!("Dimension: {:?}", ws.dimension);
            println!("Rows: {}", ws.rows.len());
            
            for (r, row) in ws.rows.iter().enumerate().take(5) {
                print!("Row {}: ", r);
                for cell in &row.cells {
                    print!("[{}:{}]={:?} ", cell.reference.col, cell.reference.row, cell.value);
                }
                println!();
            }
        }
    }
}
