# Content Branch

This branch contains personal CV data and content for the CV generator.

⚠️ **DO NOT CREATE PULL REQUESTS FROM THIS BRANCH** ⚠️

This is a data-only branch that is never meant to be merged into main.

## Contents

- `data/cv_data.json` - Personal CV information
- Future: Blog posts, images, and other content

## How it works

1. The main branch contains the CV generator code
2. This content branch contains personal data
3. During CI/CD, data from this branch is fetched and used to build the site
4. The branches remain separate - code in main, data in content

## Updating CV Data

To update your CV information:

```bash
git checkout content
# Edit data/cv_data.json
git add data/cv_data.json
git commit -m "Update CV data"
git push origin content
```

The next deployment of the main branch will automatically use the updated data.
