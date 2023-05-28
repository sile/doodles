/**
 * pixcil
 * Pixel art editor
 * @version: 0.3.0
 * @author: Takeru Ohta
 * @license: (MIT OR Apache-2.0)
 **/

(function (factory) {
    typeof define === 'function' && define.amd ? define(factory) :
    factory();
})((function () { 'use strict';

    const CACHE_NAME = "pixcil-9e1c6e45-337d-43fb-a369-43ef65465100";
    // @ts-ignore
    self.addEventListener("install", (e) => {
        console.log("[Service Worker] Install");
        // @ts-ignore
        e.waitUntil(self.skipWaiting());
    });
    self.addEventListener("fetch", (e) => {
        // @ts-ignore
        if (!e.request.url.startsWith("https://")) {
            // @ts-ignore
            e.respondWith(fetch(e.request));
            return;
        }
        // @ts-ignore
        e.respondWith(
        // @ts-ignore
        caches.match(e.request).then((r) => {
            if (r) {
                return Promise.resolve(r);
            }
            // @ts-ignore
            console.log("[Service Worker] Fetching resource: " + e.request.url);
            // @ts-ignore
            return fetch(e.request).then((response) => {
                return caches.open(CACHE_NAME).then((cache) => {
                    // @ts-ignore
                    console.log("[Service Worker] Caching new resource: " + e.request.url);
                    // @ts-ignore
                    cache.put(e.request, response.clone());
                    return response;
                });
            });
        }));
    });
    self.addEventListener("activate", (e) => {
        console.log("[Service Worker] Activate");
        // @ts-ignore
        e.waitUntil(caches.keys().then((keyList) => {
            return Promise.all(keyList.map((key) => {
                if (key !== CACHE_NAME) {
                    console.log("[Service Worker] Delete old cache: " + key);
                    return caches.delete(key);
                }
            }));
        }));
    });

}));
