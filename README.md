# OsmAnd Live Tracker Web

**This project still needs more testing and polish. Use with caution**

This is a web application designed to be used with OsmAnd's live tracking feature.

It simply receives GET requests with URL parameters that contain the location information, and then stores it in a DB and serves it on a simple web map.

Authentication is simply through a token that is a URL parameter, and it needs to match the token that was used as an environment variable for the server.

## OsmAnd Setup

In the settings for the app, select the profile you're going to use, then go to Trip Recording -> Online Tracking.
Enter the following URL, with the domain pointing to where you've hosted the server and the token set to a random value that is also use on the server: `https://example.com/log?lat={0}&lon={1}&timestamp={2}&hdop={3}&altitude={4}&speed={5}&bearing={6}&token=foobar`.
Set the tracking interval to the value you'd like, and then you're ready!
