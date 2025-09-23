# GitHub Pages Deployment Guide

This guide will help you deploy your CV site to GitHub Pages with a custom domain.

## 🚀 Quick Setup

### 1. GitHub Repository Setup

1. **Push your code to GitHub**:
   ```bash
   git add .
   git commit -m "Add GitHub Pages deployment setup"
   git push origin main
   ```

2. **Enable GitHub Pages**:
   - Go to your repository on GitHub
   - Navigate to `Settings` → `Pages`
   - Under "Source", select "GitHub Actions"
   - The workflow will automatically trigger

### 2. Custom Domain Configuration

1. **Update CNAME file**:
   - Edit `static/CNAME` and replace `yourdomain.com` with your actual domain
   - Example: `cv.yourname.com` or `www.yourname.com`

2. **DNS Configuration**:
   Add these DNS records with your domain provider:

   **For a subdomain (recommended)**:
   ```
   Type: CNAME
   Name: cv (or www)
   Value: yourusername.github.io
   ```

   **For apex domain**:
   ```
   Type: A
   Name: @ (or leave blank)
   Value: 185.199.108.153
   Value: 185.199.109.153
   Value: 185.199.110.153
   Value: 185.199.111.153
   ```

3. **GitHub Pages Domain Setup**:
   - In your repository: `Settings` → `Pages`
   - Under "Custom domain", enter your domain
   - Check "Enforce HTTPS" (recommended)

## 🔧 How It Works

### Automatic Deployment
- The GitHub Actions workflow builds your site on every push to `main`
- It generates the static files using `cargo run --bin cv`
- Deploys the `dist/` folder to GitHub Pages

### Build Process
1. **Rust Environment**: Sets up Rust toolchain and caches dependencies
2. **Typst Installation**: Installs Typst for PDF generation
3. **Site Generation**: Runs your Rust application to generate static files
4. **GitHub Pages**: Uploads and deploys the generated site

## 📁 Files Created

- `.github/workflows/deploy.yml` - GitHub Actions workflow
- `static/CNAME` - Custom domain configuration
- `DEPLOYMENT.md` - This deployment guide

## 🔒 Security Features

Your site includes several security enhancements:
- HTTPS enforcement (when using custom domain)
- Security headers in `.htaccess` and `_headers` files
- CSP (Content Security Policy) headers
- Performance optimizations

## 🌐 DNS Propagation

After setting up DNS:
- Changes typically take 5-60 minutes
- Some providers may take up to 24 hours
- You can check DNS propagation at: https://dnschecker.org

## 🐛 Troubleshooting

### Build Fails
- Check the Actions tab in your GitHub repository
- Ensure all required files are committed
- Verify Cargo.toml has the correct binary name

### Domain Not Working
- Verify DNS records are correct
- Check that CNAME file contains only your domain name
- Ensure HTTPS is enforced in GitHub Pages settings

### Site Not Updating
- Check that the workflow completed successfully
- Try triggering a manual deployment via Actions tab
- Clear your browser cache

## 📝 Updating Your Site

To update your site:
1. Make changes to your content (data files, templates, etc.)
2. Commit and push to main branch
3. GitHub Actions will automatically rebuild and deploy

## 💡 Tips

- **Blog Posts**: Add new markdown files to `data/blog/posts/`
- **CV Updates**: Modify `data/cv_data.json`
- **Styling**: Update CSS files in `static/css/`
- **Custom Domain**: Remember to update the CNAME file when changing domains

## 🔗 Useful Links

- [GitHub Pages Documentation](https://docs.github.com/en/pages)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Custom Domain Setup](https://docs.github.com/en/pages/configuring-a-custom-domain-for-your-github-pages-site)