# UMass Amherst Web Ring

This is a web ring for UMass Amherst students, alumni, and faculty.
It is a collection of personal websites, blogs, and other web pages.

## How to join
1. Add your website to the members.toml and submit a pull request.
2. Once accepted make sure to add the webring to your website in some way.

### Using our script
Add the following script to your website:
```html
<script id="umaring_js" src="https://umaring.mkr.cx/ring.js?id=ID"></script>
<div id="umaring"></div>
```

Replace `ID` with your ID in the members.toml file.
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
Please pull the latest version of the webring at least once every 6 hours, but preferably once every hour. This will ensure that the webring is up to date.

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
