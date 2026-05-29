import { createRouter, createWebHistory } from "vue-router";
import { authGuard } from "@auth0/auth0-vue";
import HomeView from "@/views/HomeView.vue";
import MemberView from "@/views/MemberView.vue";
import CallbackView from "@/views/CallbackView.vue";

export const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: "/", name: "home", component: HomeView },
    { path: "/callback", name: "callback", component: CallbackView },
    { path: "/app", name: "member", component: MemberView, beforeEnter: authGuard },
  ],
});
