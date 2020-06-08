import Vue from 'vue'
import App from './App.vue'

import { WritableStream } from "web-streams-polyfill/ponyfill/es6";
import streamSaver from "streamsaver";
if (!window.WritableStream) {
  streamSaver.WritableStream = WritableStream
}
streamSaver.mitm = new URL(window.location);
streamSaver.mitm.pathname = 'mitm.html'
streamSaver.mitm.search = 'version=2.0.0'
window.streamSaver = streamSaver;

import { library } from "@fortawesome/fontawesome-svg-core";
import { faFile, faFileArchive } from "@fortawesome/free-solid-svg-icons";
library.add(faFile);
library.add(faFileArchive);

import { FontAwesomeIcon } from "@fortawesome/vue-fontawesome";
Vue.component('font-awesome-icon', FontAwesomeIcon);

Vue.config.productionTip = false

new Vue({
  render: h => h(App),
}).$mount('#app')
