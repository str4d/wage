import Vue from 'vue'
import App from './App.vue'

import { WritableStream } from "web-streams-polyfill/ponyfill/es6";
import streamSaver from "streamsaver";
if (!window.WritableStream) {
  streamSaver.WritableStream = WritableStream
}
streamSaver.mitm = new URL(window.location);
streamSaver.mitm.pathname = 'mitm.html'
streamSaver.mitm.search = `version=${streamSaver.version.full}`
window.streamSaver = streamSaver;

import { library } from "@fortawesome/fontawesome-svg-core";
import {
  faFile,
  faFileAlt,
  faFileArchive,
  faFileAudio,
  faFileCode,
  faFileCsv,
  faFileExcel,
  faFileImage,
  faFilePdf,
  faFilePowerpoint,
  faFileVideo,
  faFileWord,
} from "@fortawesome/free-solid-svg-icons";
library.add(faFile);
library.add(faFileAlt);
library.add(faFileArchive);
library.add(faFileAudio);
library.add(faFileCode);
library.add(faFileCsv);
library.add(faFileExcel);
library.add(faFileImage);
library.add(faFilePdf);
library.add(faFilePowerpoint);
library.add(faFileVideo);
library.add(faFileWord);

import { FontAwesomeIcon } from "@fortawesome/vue-fontawesome";
Vue.component('font-awesome-icon', FontAwesomeIcon);

Vue.config.productionTip = false

new Vue({
  render: h => h(App),
}).$mount('#app')
