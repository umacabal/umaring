# UMass Amherst Web Ring

This is a web ring for UMass Amherst students, alumni, and faculty.
It is a collection of personal websites, blogs, and other web pages.

## How to join
1. Add your website to the members.toml and submit a pull request.
2. Once accepted make sure to add the webring to your website in some way.
3. The webring will scan your website every ~30 minutes to check for the webring. We do this to avoid having broken links in the chain. If you believe your site is setup properly but is not showing up in the webring make an issue.

### Using our script
> [!Warning]  
> If your website is powered by a frontend framework like Svelte, Vue, or React using the script might have strange results. It is suggested to write the integration yourself with the HTTP API indicated below. It's not scary, it's just a `fetch` call.

Add the following script to your website:
```html
<script id="umaring_js" src="https://umaring.mkr.cx/ring.js?id=USERNAME"></script>
<div id="umaring"></div>
```

Replace `USERNAME` with your id from the members.toml file.
Make sure to keep in the `id="umaring_js"` part of the script tag.

### Building your own integration
Please integrate with the following API:
`GET https://umaring.mkr.cx/:id`
This will return a JSON object with the following format:
```json
{
    "prev": {
        "id":"usera",
        "name":"User A",
        "url":"https://usera.com"
    },
    "member": {
        "id":"userb",
        "name":"User b",
        "url":"https://userb.com"
    },
    "next": {
        "id":"userc",
        "name":"User C",
        "url":"https://userc.com"
    }
}
```
If values are being cached on a backend please pull the latest version of the webring at least once every hour, but preferably once every five minutes. 
This will ensure that the webring is up to date.
If this request is being done of the clients you can just do the GET every page load, optionally caching in user local storage.


## 88x31 Button
If you want to add a button to your website, you can use the following image:
![UMass Amherst Web Ring](umass.png)

You should turn off anti-aliasing for the image to make it look better.
```
img {
    image-rendering: auto;
    image-rendering: crisp-edges;
    image-rendering: pixelated;
    image-rendering: -webkit-optimize-contrast;
}
```
