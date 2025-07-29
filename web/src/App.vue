
<template>

  <!-- Header -->
  <div class="container mx-auto max-w-3xl relative">
    <div class="flex items-center mt-5">
      <div class="inline-block float-left ml-3 flex-1 min-w-0">
        <h1 class="text-4xl font-medium truncate">
          <img alt="" class="inline align-middle" src="/favicon.ico" width="32"/>
          <span class="align-middle">WireGuard Management Console</span>
        </h1>
      </div>

      <div class="inline-block float-right px-3 whitespace-nowrap align-middle">
        <div v-if="requiresPassword" class="relative mb-5 bg-blue-50">
          <div class="text-sm text-gray-400 cursor-pointer hover:underline absolute top-0 right-0" @click="">
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
          <div v-if="webServerStatus === ServerStatusEnum.unknown"
               class="inline-block shadow-md rounded-lg bg-yellow-500 hover:bg-yellow-400 p-1.5 mr-0.5"
               title="Management Web Server Status Unknown"></div>
          <div v-else-if="webServerStatus === ServerStatusEnum.down"
               class="inline-block shadow-md rounded-lg bg-red-500 hover:bg-red-400 p-1.5 mr-0.5"
               title="Management Web Server is Down/Not reachable"></div>
          <div v-else-if="webServerStatus === ServerStatusEnum.up"
               class="inline-block shadow-md rounded-lg bg-green-500 hover:bg-green-400 p-1.5 mr-0.5"
               title="Management Web Server is Up"></div>
          <span class="text-sm text-gray-500">Web Server Status</span>
        </div>
        <div>
          <div v-if="wireguardStatus === ServerStatusEnum.unknown"
               class="inline-block align-middle shadow-md rounded-full w-5 h-3 bg-yellow-500 hover:bg-yellow-400 transition-all"
               title="WireGuard Networking Status Unknown">
            <div class="shadow-md rounded-full w-1 h-1 m-1 ml-2 bg-white"></div>
          </div>
          <div v-else-if="wireguardStatus === ServerStatusEnum.down"
               class="inline-block align-middle shadow-md rounded-full w-5 h-3 mr-0.25 bg-red-500 cursor-pointer hover:bg-red-400 transition-all"
               title="Enable WireGuard Networking"
               @click="dialogId = 'network-toggle'">
            <div class="shadow-md rounded-full w-1 h-1 m-1 bg-white"></div>
          </div>
          <div v-else-if="wireguardStatus === ServerStatusEnum.up"
               class="inline-block align-middle shadow-md rounded-full w-5 h-3 bg-green-500 cursor-pointer hover:bg-green-400 transition-all"
               title="Disable WireGuard Networking"
               @click="dialogId = 'network-toggle'">
            <div class="shadow-md rounded-full w-1 h-1 m-1 ml-3 bg-white"></div>
          </div>
          <span class="text-sm text-gray-500">WireGuard Status</span>
        </div>
      </div>
    </div>

    <div class="flex items-center opacity-50">

      <div v-if="network.this_peer" class="inline-block mr-auto text-gray-800 text-xs ml-5">
        <span>Network Identifier: <strong class="text-sm inline-block">{{ network.identifier }}</strong></span>
      </div>

      <div v-if="network.this_peer"
           class=" inline-block overflow-auto ml-auto mr-3 text-gray-800 text-xs flex-0 text-right">
        <span>
          Host: <strong class="text-sm inline-block whitespace-pre-wrap">{{
            network.peers[network.this_peer].name
          }}</strong>
        </span>
        <span class="inline-block whitespace-pre-wrap">
          ({{ network.this_peer }})
        </span>
        <span class="inline-block whitespace-pre-wrap">
          @
          {{ network.peers[network.this_peer].address }} /
          {{ network.peers[network.this_peer].endpoint.value }}
        </span>
      </div>
    </div>
  </div>

  <!-- Map -->
  <div class="container mx-auto mt-3 max-w-6xl">
    <map-visual :network="network"
                class="shadow-md rounded-lg bg-white overflow-hidden mx-3 my-2 justify-center"
                style="max-height: 80vh"
                @peer-selected="onPeerSelected"></map-visual>
  </div>

  <!-- Add a Peer Buttons -->
  <div class="container mx-auto max-w-6xl">
    <!-- Add a Peer -->
    <div class="items-center justify-center p-3 px-5 border-gray-100">
      <button :disabled="webServerStatus !== ServerStatusEnum.up"
              class="bg-gray-200 text-gray-700 border-2 border-gray-500 py-2 px-4 rounded items-center transition w-full enabled:hover:bg-green-700 enabled:hover:border-green-700 disabled:bg-gray-400 disabled:border-gray-400 enabled:hover:text-white"
              @click="dialogId = 'create-peer'">
        <span class="text-sm">Add a Peer</span>
      </button>
    </div>
  </div>

  <!-- Footer -->
  <footer class="text-center text-gray-500 my-5 font-mono mx-2">
    <div v-if="version">
      <small :title="version.readable_datetime" class="inline-block whitespace-pre-wrap">
        backend: <strong>{{ version.backend }}</strong>
      </small>
      <small :title="version.readable_datetime" class="inline-block whitespace-pre-wrap">
        frontend: <strong>{{ version.frontend }}</strong>
      </small>
      <small :title="version.readable_datetime" class="inline-block whitespace-pre-wrap">
        built: <strong>{{ version.built }}</strong>
      </small>
      <small :title="last_fetch.readable" class="inline-block whitespace-pre-wrap">
        last fetched: <strong>{{ last_fetch.rfc3339 }}</strong>
      </small>
    </div>

    <small>&copy; Copyright 2024-2025, <a class="hover:underline" href="https://yasar.idikut.cc/">Yaşar
      İdikut</a></small>
  </footer>

  <!-- Dialog: WireGuard Enable/Disable -->
  <custom-dialog v-if="dialogId === 'network-toggle'" :left-button-click="() => { dialogId = '' }"
                 :left-button-text="'Cancel'"
                 :right-button-classes="wireguardStatus === ServerStatusEnum.up ? ['text-white', 'bg-red-600', 'hover:bg-red-700'] : ['text-white', 'bg-green-600', 'hover:bg-green-700']"
                 :right-button-click="() => { toggleWireGuardNetworking(); dialogId = ''; }"
                 :right-button-text="wireguardStatus === ServerStatusEnum.up ? 'Disable' : 'Enable'"
                 class="z-10"
                 icon="danger">
    <h3 class="text-lg leading-6 font-medium text-gray-900">
      {{ wireguardStatus === ServerStatusEnum.up ? 'Disable' : 'Enable' }} the WireGuard Network
    </h3>
    <div class="mt-2 text-sm text-gray-500">
      Are you sure you want to {{ wireguardStatus === ServerStatusEnum.up ? 'disable' : 'enable' }} the WireGuard
      network?
    </div>
  </custom-dialog>

  <!-- Dialog: Peer View/Edit -->
  <peer-config-window v-if="dialogId.startsWith('selected-peer-id=')"
                      v-model:dialog-id="dialogId"
                      :network="network"
                      :peer-id="dialogId.slice(17, dialogId.length)"></peer-config-window>

  <!-- Dialog: Peer Create -->
  <peer-create-window v-if="dialogId === 'create-peer'"
                      v-model:dialog-id="dialogId"
                      :network="network"></peer-create-window>

</template>

<script>
import {initFlowbite} from 'flowbite'

import API from "./js/api.js";
import MapVisual from "./components/map-visual.vue";
import CustomDialog from "./components/custom-dialog.vue";
import PeerConfigWindow from "./components/peer-config-window.vue";
import PeerCreateWindow from "./components/peer-create-window.vue";

import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';

dayjs.extend(relativeTime);

export default {
  name: "app",
  components: {MapVisual, CustomDialog, PeerConfigWindow, PeerCreateWindow},
  data() {
    return {
      refreshRate: 1000,
      webServerStatus: 0,
      wireguardStatus: 0,
      ServerStatusEnum: {
        'unknown': 0,
        'down': 1,
        'up': 2
      },
      requiresPassword: false,
      dialogId: '',
      network: {},
      digest: '',
      version: null,
      last_fetch: {
        rfc3339: "",
        readable: "",
      },

    }
  },
  mounted: function () {
    initFlowbite();

    setInterval(() => {
      this.refresh()
    }, this.refreshRate)

    API.get_version().then(response => {
      const last_built_date = (new Date(Date.parse(response.built)))
      this.version = {
        backend: response.backend,
        frontend: response.frontend,
        built: response.built,
        readable_datetime: `${last_built_date} [${dayjs(last_built_date).fromNow()}]`
      }
    });
  },
  computed: {},
  methods: {
    async refresh() {
      let need_to_update_network = true;
      if (this.digest.length === 64) {
        await API.get_summary('?only_digest=true').then(summary => {
          this.webServerStatus = this.ServerStatusEnum.up;
          this.wireguardStatus = summary.status;
          need_to_update_network = this.digest !== summary.digest;

          this.last_fetch.rfc3339 = summary.timestamp;
          const last_fetch_date = (new Date(Date.parse(this.last_fetch.rfc3339)))
          this.last_fetch.readable = `${last_fetch_date} [${dayjs(last_fetch_date).fromNow()}]`;
        }).catch(err => {
          this.wireguardStatus = this.ServerStatusEnum.unknown;
          if (err.toString() === 'TypeError: Load failed') {
            this.webServerStatus = this.ServerStatusEnum.down;
          } else {
            this.webServerStatus = this.ServerStatusEnum.unknown;
            console.log('getNetwork error =>');
            console.log(err);
          }
        });
      }

      if (!need_to_update_network) {
        return;
      }

      await API.get_summary('?only_digest=false').then(summary => {
        this.webServerStatus = this.ServerStatusEnum.up;
        this.digest = summary.digest;
        this.network = summary.network;
        this.network.static_peer_ids = [];
        this.network.roaming_peer_ids = [];
        Object.entries(summary.network.peers).forEach(([peerId, peerDetails]) => {
          if (peerDetails.endpoint.enabled) {
            this.network.static_peer_ids.push(peerId);
          } else {
            this.network.roaming_peer_ids.push(peerId);
          }
        })
        this.wireguardStatus = summary.status

        this.last_fetch.rfc3339 = summary.timestamp;
        const last_fetch_date = (new Date(Date.parse(this.last_fetch.rfc3339)))
        this.last_fetch.readable = `${last_fetch_date} [${dayjs(last_fetch_date).fromNow()}]`;
      }).catch(err => {
        this.wireguardStatus = this.ServerStatusEnum.unknown;
        if (err.toString() === 'TypeError: Load failed') {
          this.webServerStatus = this.ServerStatusEnum.down;
        } else {
          this.webServerStatus = this.ServerStatusEnum.unknown;
          console.log('getNetwork error =>');
          console.log(err);
        }
      });
    },
    toggleWireGuardNetworking() {
      console.log(`${this.wireguardStatus === this.ServerStatusEnum.up ? 'Disabling' : 'Enabling'} WireGuard Network...`)
      // TODO: implement me
    },
    onPeerSelected(peer_id) {
      this.dialogId = `selected-peer-id=${peer_id}`;
    }
  }
}
</script>

