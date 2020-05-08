import Vue from 'vue'
import App from './App.vue'

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
