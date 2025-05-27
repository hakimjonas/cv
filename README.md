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

## GitHub Integration

This project can fetch your GitHub repositories and include them in your CV. To use this feature:

1. Add your GitHub username and/or organization to the `github_sources` field in `data/cv_data.json`:
   ```json
   {
     "github_sources": [
       {
         "username": "yourusername",
         "organization": null
       },
       {
         "username": null,
         "organization": "yourorganization"
       }
     ]
   }
   ```

2. **Important: GitHub API Rate Limiting**

   The GitHub API has rate limits:
   - Unauthenticated requests: 60 requests per hour
   - Authenticated requests: 5,000 requests per hour

   To avoid rate limiting, you can provide a GitHub API token using one of the following methods (in order of preference):

   a. Create a personal access token on GitHub:
      - Go to GitHub Settings → Developer settings → Personal access tokens
      - Generate a new token with the `public_repo` scope

   b. Store the token in your git config (recommended for local development):
      ```bash
      git config --global cv.github.token "your-token-here"
      ```
      This stores the token securely in your global git config, which is not committed to version control.

   c. Set the token as an environment variable:
      ```bash
      GITHUB_TOKEN=your-token-here cargo run
      ```

3. **For Production Deployment with GitHub Actions**

   When deploying with GitHub Actions, you can use GitHub Secrets to store your token securely:

   a. Add your token as a secret in your GitHub repository:
      - Go to your repository → Settings → Secrets and variables → Actions
      - Click "New repository secret"
      - Name: `GITHUB_TOKEN`
      - Value: your personal access token

   b. In your GitHub Actions workflow file, make the secret available as an environment variable:
      ```yaml
      jobs:
        build:
          runs-on: ubuntu-latest
          env:
            GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          steps:
            - uses: actions/checkout@v3
            - name: Build CV
              run: cargo run --release
      ```

   The application will automatically detect when it's running in GitHub Actions and use the token from the environment.

## Dependencies

- [serde](https://serde.rs/): For JSON serialization/deserialization
- [askama](https://github.com/djc/askama): For HTML templating
- [im](https://docs.rs/im/): For immutable data structures
- [anyhow](https://docs.rs/anyhow/): For error handling
- [reqwest](https://github.com/seanmonstar/reqwest): For HTTP requests
- [tokio](https://tokio.rs/): For async runtime
- [typst-cli](https://github.com/typst/typst): For PDF generation

## License

This project is licensed under the MIT License - see the LICENSE file for details.
