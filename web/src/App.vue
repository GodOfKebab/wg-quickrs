
<template>

  <div class="container mx-auto max-w-3xl">

    <!-- Header -->
    <div class="mt-10 mb-2" style="display: flex; align-items: center;">
      <div class="inline-block float-left ml-3" style="flex: 1; min-width: 0;">
        <h1 class="text-4xl font-medium truncate">
          <img class="inline align-middle" src="/favicon.ico" width="32"/>
          <span class="align-middle">WireGuard Management Console</span>
        </h1>
      </div>

      <div class=" inline-block float-right p-3 whitespace-nowrap bg-gray-50 align-middle">
        <div v-if="requiresPassword" class="relative mb-5 bg-blue-50">
          <div class="text-sm text-gray-400 cursor-pointer hover:underline absolute top-0 right-0" @click="logout">
            Logout
            <svg class="h-3 inline" fill="none" stroke="currentColor" viewBox="0 0 24 24"
                 xmlns="http://www.w3.org/2000/svg">
              <path d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"
                    stroke-linecap="round" stroke-linejoin="round"
                    stroke-width="2"/>
            </svg>
          </div>
        </div>
        <div>
          <div v-if="webServerStatus === 'unknown'"
               class="inline-block shadow-md rounded-lg bg-yellow-500 hover:bg-yellow-400 p-1.5 mr-0.5"
               title="Management Web Server Status Unknown"></div>
          <div v-if="webServerStatus === 'down'"
               class="inline-block shadow-md rounded-lg bg-red-500 hover:bg-red-400 p-1.5 mr-0.5"
               title="Management Web Server is Down/Not reachable"></div>
          <div v-if="webServerStatus === 'up'"
               class="inline-block shadow-md rounded-lg bg-green-500 hover:bg-green-400 p-1.5 mr-0.5"
               title="Management Web Server is Up"></div>
          <span class="text-sm text-gray-500">Web Server Status</span>
        </div>
        <div>
          <div v-if="wireguardStatus === 'unknown'"
               class="inline-block align-middle shadow-md rounded-full w-5 h-3 bg-yellow-500 hover:bg-yellow-400 transition-all"
               title="WireGuard Networking Status Unknown">
            <div class="shadow-md rounded-full w-1 h-1 m-1 ml-2 bg-white"></div>
          </div>
          <div v-if="wireguardStatus === 'up'"
               class="inline-block align-middle shadow-md rounded-full w-5 h-3 bg-green-500 cursor-pointer hover:bg-green-400 transition-all"
               title="Disable WireGuard Networking"
               @click="dialogId = 'network-toggle'">
            <div class="shadow-md rounded-full w-1 h-1 m-1 ml-3 bg-white"></div>
          </div>
          <div v-if="wireguardStatus === 'down'"
               class="inline-block align-middle shadow-md rounded-full w-5 h-3 mr-0.25 bg-red-500 cursor-pointer hover:bg-red-400 transition-all"
               title="Enable WireGuard Networking"
               @click="dialogId = 'network-toggle'">
            <div class="shadow-md rounded-full w-1 h-1 m-1 bg-white"></div>
          </div>
          <span class="text-sm text-gray-500">WireGuard Status</span>
        </div>
      </div>
    </div>

    <!-- Map -->
    <div id="graph" class="shadow-md rounded-lg bg-white overflow-hidden mx-3 my-2 justify-center h-96"></div>

    <!-- Add a Static/Roaming Peer -->
    <div class="grid grid-cols-2 gap-2">
      <div v-for="mobility in ['static', 'roaming']"
           class="grid-cols-1 flex flex-row flex-auto items-center justify-center p-3 px-5 border-gray-100">
        <button :disabled="webServerStatus !== 'up'"
                class="bg-gray-200 enabled:hover:bg-green-700 enabled:hover:border-green-700 disabled:bg-gray-400 disabled:border-gray-400 enabled:hover:text-white text-gray-700 border-2 border-gray-500 py-2 px-4 rounded inline-flex items-center transition"
                @click=""> <!-- TODO: open peer create window-->
          <span class="text-sm">Add a {{ mobility[0].toUpperCase() + mobility.slice(1) }} Peer</span>
        </button>
      </div>
    </div>

    <!-- Dialog: WireGuard Enable/Disable -->
    <custom-dialog v-if="dialogId === 'network-toggle'" :left-button-click="() => { dialogId = null }"
                   :left-button-text="'Cancel'"
                   :right-button-classes="wireguardStatus === 'up' ? ['text-white', 'bg-red-600', 'hover:bg-red-700'] : ['text-white', 'bg-green-600', 'hover:bg-green-700']"
                   :right-button-click="() => { toggleWireGuardNetworking(); dialogId = null; }"
                   :right-button-text="wireguardStatus === 'up' ? 'Disable' : 'Enable'"
                   class="z-10"
                   icon="danger">
      <h3 class="text-lg leading-6 font-medium text-gray-900">
        {{ wireguardStatus === 'up' ? 'Disable' : 'Enable' }} the WireGuard Network
      </h3>
      <div class="mt-2 text-sm text-gray-500">
        Are you sure you want to {{ wireguardStatus === 'up' ? 'disable' : 'enable' }} the WireGuard network?
      </div>
    </custom-dialog>
  </div>

  <footer class="text-center text-gray-500 my-10">
    <small>&copy; Copyright 2024, <a class="hover:underline" href="https://yasar.idikut.cc/">Yaşar İdikut</a></small>
  </footer>
</template>

<script>
import CustomDialog from "./components/custom-dialog.vue";

import API from "./js/api.js";

export default {
  name: "app",
  components: {CustomDialog},
  data() {
    return {
      refreshRate: 1000,
      api: null,
      webServerStatus: 'up',
      wireguardStatus: 'up',
      requiresPassword: false,
      dialogId: null,
    }
  },
  mounted: function () {
    this.api = new API();
    setInterval(() => {
      this.refresh()
    }, this.refreshRate)
  },
  computed: {},
  methods: {
    async refresh() {
      // Get WirGuard Server Status
      await this.api.getWireGuardStatus().then(wgStatus => {
        this.webServerStatus = 'up';
        if (wgStatus['status'] === 'up') {
          this.wireguardStatus = 'up';
        } else if (wgStatus['status'] === 'down') {
          this.wireguardStatus = 'down';
        }
      }).catch(() => {
        this.webServerStatus = 'down';
        this.wireguardStatus = 'unknown';
      });
      if (this.wireguardStatus !== 'up') return null;

      // TODO: implement me
    },
    toggleWireGuardNetworking() {
      console.log(`${this.wireguardStatus === 'up' ? 'Disabling' : 'Enabling'} WireGuard Network...`)
      // TODO: implement me
    },
  }
}
</script>

