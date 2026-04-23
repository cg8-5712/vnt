import { createRouter, createWebHashHistory } from 'vue-router'

import ConfigView from '@/views/ConfigView.vue'
import OverviewView from '@/views/OverviewView.vue'
import PeersView from '@/views/PeersView.vue'
import RoutesView from '@/views/RoutesView.vue'

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: '/',
      redirect: '/overview',
    },
    {
      path: '/overview',
      name: 'overview',
      component: OverviewView,
    },
    {
      path: '/configs',
      name: 'configs',
      component: ConfigView,
    },
    {
      path: '/peers',
      name: 'peers',
      component: PeersView,
    },
    {
      path: '/routes',
      name: 'routes',
      component: RoutesView,
    },
  ],
})
