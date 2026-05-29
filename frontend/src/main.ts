import { createApp } from "vue";
import { createPinia } from "pinia";
import "./style.css";
import App from "./App.vue";
import { router } from "./router";
import { auth0 } from "./auth";

createApp(App).use(createPinia()).use(router).use(auth0).mount("#app");
