import Vue from 'vue'
import App from './App.vue'

import streamSaver from "streamsaver";
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
  faUpload,
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
library.add(faUpload);

import { FontAwesomeIcon } from "@fortawesome/vue-fontawesome";
Vue.component('font-awesome-icon', FontAwesomeIcon);

import Buefy from 'buefy'
import 'buefy/dist/buefy.css'
Vue.use(Buefy, {
  defaultIconComponent: 'font-awesome-icon',
  defaultIconPack: 'fas',
});

Vue.config.productionTip = false

new Vue({
  render: h => h(App),
}).$mount('#app')
