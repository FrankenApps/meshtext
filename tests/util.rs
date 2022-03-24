/// Dumps the vertex data into a csv file.
///
/// Arguments:
///
/// * `path`: The output path to which the file will be written.
/// * `is_3d`: Wether the given _vertices_ use 3D positions (XYZ) or 2D positions (XY).
/// * `data`: The vertex positions of a glyph or text section.
#[allow(unused)]
pub(crate) fn write_vertices_to_csv(
    path: &str,
    is_3d: bool,
    data: &Vec<f32>,
) -> Result<(), std::io::Error> {
    let mut content = String::new();
    if is_3d {
        // Write the header.
        content.push_str("x, y, z\n");

        // Write the data.
        let rows: String = data
            .chunks(3)
            .map(|c| format!("{}, {}, {}\n", c[0], c[1], c[2]))
            .collect();
        content.push_str(rows.as_str());
    } else {
        // Write the header.
        content.push_str("x, y\n");

        // Write the data.
        let rows: String = data
            .chunks(2)
            .map(|c| format!("{}, {}\n", c[0], c[1]))
            .collect();
        content.push_str(rows.as_str());
    }

    std::fs::write(path, content)?;
    Ok(())
}

/// Creates a plot of the generated vertices.
///
/// Arguments:
///
/// * `path`: The output path to which the file will be written.
/// * `is_3d`: Wether the given _vertices_ use 3D positions (XYZ) or 2D positions (XY).
/// * `data`: The vertex positions of a glyph or text section.
#[allow(unused)]
pub(crate) fn plot_vertices(
    path: &str,
    is_3d: bool,
    data: &Vec<f32>,
) -> Result<(), Box<dyn std::error::Error>> {
    use plotters::prelude::*;

    let root = BitMapBackend::new(path, (1024, 1024)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption(format!("Vertices and Triangles"), ("sans-serif", 70))
        .build_cartesian_2d(-0.25..0.75, -0.25..0.75)?;

    // Get triangles from vertices.
    let mut triangles: Vec<Vec<(f64, f64)>> = Vec::new();
    let (vertices_text, triangles_text);
    if is_3d {
        for t in data.chunks(9) {
            triangles.push(vec![
                (t[0] as f64, t[1] as f64),
                (t[3] as f64, t[4] as f64),
                (t[6] as f64, t[7] as f64),
            ]);
        }
        vertices_text = format!("Vertices: {}", data.len() / 3);
    } else {
        for t in data.chunks(6) {
            triangles.push(vec![
                (t[0] as f64, t[1] as f64),
                (t[2] as f64, t[3] as f64),
                (t[4] as f64, t[5] as f64),
            ]);
        }
        vertices_text = format!("Vertices: {}", data.len() / 2);
    }
    triangles_text = format!("Triangles: {}", triangles.len());

    root.draw_text(
        vertices_text.as_str(),
        &("sans-serif", 30).into_text_style(&root).color(&BLACK),
        (20, 950),
    )?;
    root.draw_text(
        triangles_text.as_str(),
        &("sans-serif", 30).into_text_style(&root).color(&BLACK),
        (20, 980),
    )?;

    for triangle in triangles.iter_mut() {
        chart.draw_series(std::iter::once(Polygon::new(
            triangle.clone(),
            &RED.mix(0.2),
        )))?;
        triangle.push(triangle[0]);
        chart.draw_series(std::iter::once(PathElement::new(triangle.clone(), &RED)))?;

        chart.draw_series(
            triangle
                .iter()
                .map(|(x, y)| Circle::new((*x, *y), 4, GREEN.filled())),
        )?;
        chart.draw_series(triangle.iter().map(|(x, y)| {
            Text::new(
                format!("({:.1},{:.1})", x, y),
                (*x, *y),
                ("sans-serif", 15.0).into_font(),
            )
        }))?;
    }

    root.present().unwrap();
    Ok(())
}

/// Creates a SVG plot of the generated vertices.
///
/// Arguments:
///
/// * `path`: The output path to which the file will be written.
/// * `is_3d`: Wether the given _vertices_ use 3D positions (XYZ) or 2D positions (XY).
/// * `data`: The vertex positions of a glyph or text section.
#[allow(unused)]
pub(crate) fn plot_vertices_svg(
    path: &str,
    is_3d: bool,
    data: &Vec<f32>,
) -> Result<(), Box<dyn std::error::Error>> {
    use plotters::prelude::*;

    let root = SVGBackend::new(path, (1024, 1024)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption(format!("Vertices and Triangles"), ("sans-serif", 70))
        .build_cartesian_2d(-0.25..0.75, -0.25..0.75)?;

    // Get triangles from vertices.
    let mut triangles: Vec<Vec<(f64, f64)>> = Vec::new();
    let (vertices_text, triangles_text);
    if is_3d {
        for t in data.chunks(9) {
            triangles.push(vec![
                (t[0] as f64, t[1] as f64),
                (t[3] as f64, t[4] as f64),
                (t[6] as f64, t[7] as f64),
            ]);
        }
        vertices_text = format!("Vertices: {}", data.len() / 3);
    } else {
        for t in data.chunks(6) {
            triangles.push(vec![
                (t[0] as f64, t[1] as f64),
                (t[2] as f64, t[3] as f64),
                (t[4] as f64, t[5] as f64),
            ]);
        }
        vertices_text = format!("Vertices: {}", data.len() / 2);
    }
    triangles_text = format!("Triangles: {}", triangles.len());

    root.draw_text(
        vertices_text.as_str(),
        &("sans-serif", 30).into_text_style(&root).color(&BLACK),
        (20, 950),
    )?;
    root.draw_text(
        triangles_text.as_str(),
        &("sans-serif", 30).into_text_style(&root).color(&BLACK),
        (20, 980),
    )?;

    for triangle in triangles.iter_mut() {
        chart.draw_series(std::iter::once(Polygon::new(
            triangle.clone(),
            &RED.mix(0.2),
        )))?;
        triangle.push(triangle[0]);
        chart.draw_series(std::iter::once(PathElement::new(triangle.clone(), &RED)))?;

        chart.draw_series(
            triangle
                .iter()
                .map(|(x, y)| Circle::new((*x, *y), 4, GREEN.filled())),
        )?;
        chart.draw_series(triangle.iter().map(|(x, y)| {
            Text::new(
                format!("({:.1},{:.1})", x, y),
                (*x, *y),
                ("sans-serif", 15.0).into_font(),
            )
        }))?;
    }

    root.present().unwrap();
    Ok(())
}
