const url = `https://api.github.com/repos/zengatsu/chainwatch/releases/latest`;
const filenames = [
    "chainwatch-aarch64-apple-darwin.tar.xz",
    "chainwatch-x86_64-apple-darwin.tar.xz",
    "chainwatch-x86_64-pc-windows-msvc.zip",
    "chainwatch-aarch64-unknown-linux-gnu.tar.xz",
    "chainwatch-x86_64-unknown-linux-gnu.tar.xz",
];

async function getLatestReleaseUrls() {
    try {
        const response = await fetch(url, {
            headers: { 'Accept': 'application/vnd.github+json' }
        });

        if (!response.ok) {
            throw new Error(`GitHub API error: ${response.status} ${response.statusText}`);
        }

        const data = await response.json();

        // 1. Source Code Download URL (ZIP)
        console.log("ZIP Download URL:", data.zipball_url);

        // 2. Release Assets (Compiled binaries, installers, etc.)
        if (data.assets && data.assets.length > 0) {
            const assets = data.assets.filter(x => filenames.includes(x.name))
            assets.forEach(asset => {
                console.log(`Asset (${asset.name}):`, asset.browser_download_url);
            });
        } else {
            console.log("No compiled assets found for this release.");
        }

    } catch (error) {
        console.error("Failed to fetch release:", error.message);
    }
}

getLatestReleaseUrls();