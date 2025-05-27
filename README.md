# Personal Website with Dynamic CV Generator

This project generates a personal website with a dynamically generated CV in both HTML and PDF formats from a single Rust data source. It leverages Rust for content generation, HTML/CSS for the web interface, and Typst for PDF output.

## Features

- **Single Source of Truth**: All CV data is stored in a single JSON file
- **Multiple Output Formats**: Generates both HTML and PDF versions of your CV
- **Responsive Design**: The website works on all devices
- **Customizable**: Easy to customize the design and content
- **Functional Programming**: Uses immutable data structures and functional programming principles

## Directory Structure

```
.
├── Cargo.toml           # Rust project configuration
├── data/
│   └── cv_data.json     # CV data in JSON format
├── dist/                # Generated output files
│   ├── cv.html          # Generated HTML CV
│   ├── cv.pdf           # Generated PDF CV
│   ├── index.html       # Copied from static/
│   └── style.css        # Copied from static/
├── src/
│   ├── main.rs          # Main application entry point
│   ├── cv_data.rs       # CV data model
│   ├── html_generator.rs # HTML generation logic
│   └── typst_generator.rs # PDF generation logic
├── static/
│   ├── index.html       # Static landing page
│   └── style.css        # CSS styles for the website
└── templates/
    └── cv.html          # Askama template for CV HTML
```

## Installation

1. Install Rust and Cargo:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Install Typst CLI:
   ```bash
   cargo install typst-cli
   ```

3. Clone this repository:
   ```bash
   git clone https://github.com/yourusername/personal-website.git
   cd personal-website
   ```

4. Build the project:
   ```bash
   cargo build --release
   ```

## Usage

1. Customize your CV data in `data/cv_data.json`

2. Generate your website:
   ```bash
   cargo run --release
   ```

3. The generated files will be in the `dist/` directory:
   - `index.html`: The landing page
   - `cv.html`: The HTML version of your CV
   - `cv.pdf`: The PDF version of your CV
   - `style.css`: The CSS styles for the website

4. Deploy the contents of the `dist/` directory to your web server or hosting service

## Customization

### CV Data

Edit the `data/cv_data.json` file to update your personal information, experiences, education, skills, projects, languages, and certifications.

### HTML Templates

- `templates/cv.html`: Modify this file to change the structure of the HTML CV
- `static/index.html`: Modify this file to change the landing page
- `static/style.css`: Modify this file to change the styling of the website

### PDF Template

The PDF template is generated in code in the `src/typst_generator.rs` file. Modify the `generate_typst_markup` function to change the structure and styling of the PDF CV.

## Dependencies

- [serde](https://serde.rs/): For JSON serialization/deserialization
- [askama](https://github.com/djc/askama): For HTML templating
- [im](https://docs.rs/im/): For immutable data structures
- [anyhow](https://docs.rs/anyhow/): For error handling
- [typst-cli](https://github.com/typst/typst): For PDF generation

## License

This project is licensed under the MIT License - see the LICENSE file for details.
