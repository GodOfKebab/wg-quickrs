<template>

  <div class="flex flex-col font-mono h-screen">

    <!-- Header -->
    <div class="container mx-auto shrink-0 max-w-3xl relative">
      <!--  Logo + Name  -->
      <div class="flex mt-5">

        <!-- Middle (grows + truncates) -->
        <div class="float-left px-3 flex items-center flex-grow min-w-0">
          <div class="inline-block relative">
            <!-- Settings Button -->
            <button
                :class="[settingsDropdownOpen ? 'bg-gray-300': '']"
                class="mt-0.5 mr-2 h-8 w-8 rounded-md bg-gray-200 hover:bg-gray-300 flex items-center justify-center shrink-0"
                @click="settingsDropdownOpen = !settingsDropdownOpen">
              <img alt="settings" class="h-6" src="/icons/iconfinder/ionicons-211751_gear_icon.svg">
            </button>

            <!-- Settings Dropdown -->
            <div v-if="settingsDropdownOpen"
                 class="absolute left-0 top-9 w-24 bg-white border border-gray-200 rounded-md shadow-lg z-20 flex items-center justify-center">
              <button
                  class="block w-full text-left px-2 py-2 text-sm text-gray-700 hover:bg-gray-100 rounded-md"
                  @click="settingsDropdownOpen = false; logout();">
                <img alt="logout" class="inline-block float-left h-5 mr-1"
                     src="/icons/iconfinder/iconoir-9042719_log_out_icon.svg">
                <span>Logout</span>
              </button>
            </div>
          </div>
          <!-- Title -->
          <h1 class="text-4xl truncate font-semibold py-1">
            <span>wg-quickrs web console</span>
          </h1>
        </div>

        <!--   Indicators/Buttons   -->
        <div class="inline-block float-right pr-3 my-auto">
          <!-- Web Server Status -->
          <div class="flex items-center pl-1">
            <div v-if="webServerStatus === ServerStatusEnum.unknown"
                 class="inline-block shadow-md rounded-lg p-1.5 mr-2 bg-yellow-500 hover:bg-yellow-400"
                 title="Management Web Server Status Unknown"></div>
            <div v-else-if="webServerStatus === ServerStatusEnum.down"
                 class="inline-block shadow-md rounded-lg p-1.5 mr-2 bg-red-500 hover:bg-red-400"
                 title="Management Web Server is Down/Not reachable"></div>
            <div v-else-if="webServerStatus === ServerStatusEnum.up"
                 class="inline-block shadow-md rounded-lg p-1.5 mr-2 bg-green-500 hover:bg-green-400"
                 title="Management Web Server is Up"></div>
            <span class="text-sm text-gray-500 text-right">Web Server Status</span>
          </div>

          <!-- WireGuard Status -->
          <div class="flex items-center pl-1">
            <div v-if="wireguardStatus === ServerStatusEnum.unknown"
                 class="inline-block align-middle shadow-md rounded-full transition-all w-5 h-3 mr-1.5 bg-yellow-500 hover:bg-yellow-400"
                 title="WireGuard Networking Status Unknown">
              <div class="shadow-md rounded-full w-1 h-1 m-1 ml-2 bg-white"></div>
            </div>
            <div v-else-if="wireguardStatus === ServerStatusEnum.down"
                 class="inline-block align-middle shadow-md rounded-full transition-all w-5 h-3 mr-1.5 bg-red-500 cursor-pointer hover:bg-red-400"
                 title="Enable WireGuard Networking"
                 @click="dialogId = 'network-toggle'">
              <div class="shadow-md rounded-full w-1 h-1 m-1 bg-white"></div>
            </div>
            <div v-else-if="wireguardStatus === ServerStatusEnum.up"
                 class="inline-block align-middle shadow-md rounded-full transition-all w-5 h-3 mr-1.5 bg-green-500 cursor-pointer hover:bg-green-400"
                 title="Disable WireGuard Networking"
                 @click="dialogId = 'network-toggle'">
              <div class="shadow-md rounded-full w-1 h-1 m-1 ml-3 bg-white"></div>
            </div>
            <span class="text-sm text-gray-500 text-right" style="margin-left: 3px">WireGuard Status</span>
          </div>
        </div>
      </div>

      <!--  Network identifier and host info line(s)  -->
      <div class="flex items-center opacity-50">

        <!-- Network Identifier -->
        <div v-if="network.this_peer"
             class="inline-block text-gray-800 text-xs ml-3 text-left">
          <span>Network Identifier: <strong class="inline-block">{{ network.identifier }}</strong></span>
        </div>

        <div v-if="network.this_peer"
             class="inline-block text-gray-800 text-xs mr-3 text-right">
          <span>
            Host: <strong class="inline-block whitespace-pre-wrap">{{
              network.peers[network.this_peer].name
            }}</strong>
          </span>
          <span class="inline-block whitespace-pre-wrap">({{ network.this_peer }})</span>
          <span class="inline-block whitespace-pre-wrap">
            @
            {{ network.peers[network.this_peer].address }} /
            {{ network.peers[network.this_peer].endpoint.value }}
          </span>
        </div>
      </div>
    </div>

    <!-- Traffic Graph -->
    <div class="container mx-auto shrink-0 max-w-6xl px-3">
      <traffic-graph :network="network"
                     :telemetry="telemetry"></traffic-graph>
    </div>

    <!-- Map -->
    <div id="graph-app" class="container mx-auto flex-1 max-w-6xl mt-1 px-3 overflow-hidden">
      <map-visual :network="network"
                  :telemetry="telemetry"
                  @peer-selected="onPeerSelected"></map-visual>
    </div>

    <!-- Add a Peer Buttons -->
    <div class="container mx-auto shrink-0 max-w-6xl">
      <!-- Add a Peer -->
      <div class="items-center justify-center p-3 px-5 border-gray-100">
        <button :disabled="webServerStatus !== ServerStatusEnum.up"
                class="bg-green-100 text-gray-700 border-2 border-gray-500 py-2 px-4 rounded items-center transition w-full enabled:hover:bg-green-700 enabled:hover:border-green-700 disabled:bg-gray-400 disabled:border-gray-400 enabled:hover:text-white"
                @click="dialogId = 'create-peer'">
          <span class="text-sm">Add a Peer</span>
        </button>
      </div>
    </div>

    <!-- Footer -->
    <footer class="text-center text-gray-500 mb-5 mx-2 shrink-0">
      <small v-if="version" :title="version.readable_datetime" class="inline-block whitespace-pre-wrap">
        version:
        <strong>
          <a :href="`https://github.com/GodOfKebab/wg-quickrs/releases/tag/v${version.version}`"
             class="hover:underline"
             target="_blank">
            {{ version.version }}
          </a>
        </strong>
      </small>
      <small v-if="version" :title="version.readable_datetime" class="inline-block whitespace-pre-wrap">
        build:
        <strong>
          <a :href="`https://github.com/GodOfKebab/wg-quickrs/commits/${version.build_sha_and_date[0].split('#')[1]}`"
             class="hover:underline"
             target="_blank">
            {{ version.build_sha_and_date[0] }}
          </a>
        </strong>
        <strong>@{{ version.build_sha_and_date[1] }}</strong>
      </small>
      <small :title="last_fetch.readable" class="inline-block whitespace-pre-wrap">
        last fetched:
        <strong v-if="last_fetch.since < 0" class="text-red-700">Never</strong>
        <strong v-else-if="last_fetch.since > refreshRate * 5"
                class="text-yellow-700">{{ last_fetch.rfc3339 }}</strong>
        <strong v-else class="text-green-700">{{ last_fetch.rfc3339 }}</strong>
      </small>
      <br/>
      <br/>
      <small>
        <a class="hover:underline" href="https://www.wireguard.com/" target="_blank">
          "WireGuard" and the "WireGuard" logo are registered trademarks of Jason A. Donenfeld.
        </a>
      </small>
      <br/>
      <small>
        <span>
          © 2025
        </span>
        <strong>
          <a class="hover:underline" href="https://github.com/GodOfKebab/wg-quickrs" target="_blank">wg-quickrs</a>
        </strong>
        <span>
          -
        </span>
        <a class="hover:underline" href="https://yasar.idikut.cc/" target="_blank">
          Yaşar İdikut
        </a>
      </small>

    </footer>

    <!-- Dialog: Ask Password -->
    <password-dialog v-if="api.does_need_auth"
                     :api="api"></password-dialog>


    <custom-dialog v-if="dialogId === 'network-toggle'" :left-button-click="() => { dialogId = '' }"
                   modal-classes="max-w-xl"
                   :left-button-text="'Cancel'"
                   :right-button-classes="wireguardStatus === ServerStatusEnum.up ? ['text-white', 'bg-red-600', 'hover:bg-red-700', 'border-red-900'] : ['text-white', 'bg-green-600', 'hover:bg-green-700', 'border-green-900']"
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
    <peer-config-dialog v-if="dialogId.startsWith('selected-peer-id=')"
                        v-model:dialog-id="dialogId"
                        :api="api"
                        :network="network"
                        :peer-id="dialogId.slice(17, dialogId.length)"
                        :version="version"></peer-config-dialog>

    <!-- Dialog: Peer Create -->
    <peer-create-dialog v-if="dialogId === 'create-peer'"
                        v-model:dialog-id="dialogId"
                        :api="api"
                        :network="network"
                        :version="version"></peer-create-dialog>

  </div>
</template>

<script>
import API from "@/js/api.js";
import TrafficGraph from "@/components/traffic-graph.vue";
import MapVisual from "@/components/map-visual.vue";
import CustomDialog from "@/components/dialogs/custom-dialog.vue";
import PasswordDialog from "@/components/dialogs/password-dialog.vue";
import PeerConfigDialog from "@/components/dialogs/peer-config-dialog.vue";
import PeerCreateDialog from "@/components/dialogs/peer-create-dialog.vue";

import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';
import init from '../pkg/wg_quickrs_wasm.js';

dayjs.extend(relativeTime);

export default {
  name: "app",
  components: {
    PasswordDialog,
    TrafficGraph,
    MapVisual,
    CustomDialog,
    PeerConfigDialog,
    PeerCreateDialog
  },
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
      dialogId: '',
      network: {},
      telemetry: null,
      digest: '',
      version: null,
      last_fetch: {
        rfc3339: "",
        readable: "",
        since: -1,
      },
      wasmInitialized: false,
      api: {does_need_auth: false},
      settingsDropdownOpen: false
    }
  },
  async mounted() {
    if (!this.wasmInitialized) {
      try {
        await init();
        this.wasmInitialized = true;
      } catch (err) {
        console.error('WASM failed to load:', err);
      }
    }

    this.api = new API();
    if (localStorage.getItem('remember') === 'true') {
      this.api.token = localStorage.getItem('token') || '';
    }

    setInterval(() => {
      this.refresh()
    }, this.refreshRate)
  },
  computed: {},
  methods: {
    async refresh() {
      this.last_fetch.since = this.last_fetch.rfc3339 ? new Date() - new Date(this.last_fetch.rfc3339) : -1;

      let need_to_update_network = true;
      if (this.digest.length === 64) {
        await this.api.get_network_summary('?only_digest=true').then(summary => {
          this.webServerStatus = this.ServerStatusEnum.up;
          this.wireguardStatus = summary.status;
          need_to_update_network = this.digest !== summary.digest;
          this.telemetry = summary.telemetry;

          this.last_fetch.rfc3339 = summary.timestamp;
          const last_fetch_date = (new Date(Date.parse(this.last_fetch.rfc3339)))
          this.last_fetch.readable = `${last_fetch_date} [${dayjs(last_fetch_date).fromNow()}]`;
          this.last_fetch.since = 0;
        }).catch(err => {
          this.telemetry = null;
          this.wireguardStatus = this.ServerStatusEnum.unknown;
          if (err.toString() === 'TypeError: Load failed') {
            this.webServerStatus = this.ServerStatusEnum.down;
          } else {
            this.webServerStatus = this.ServerStatusEnum.unknown;
            console.log(err);
          }
        });
      }

      if (need_to_update_network) {
        await this.api.get_network_summary('?only_digest=false').then(summary => {
          this.webServerStatus = this.ServerStatusEnum.up;
          this.digest = summary.digest;
          this.telemetry = summary.telemetry;
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
          this.last_fetch.since = 0;
        }).catch(err => {
          this.telemetry = null;
          this.wireguardStatus = this.ServerStatusEnum.unknown;
          if (err.toString() === 'TypeError: Load failed') {
            this.webServerStatus = this.ServerStatusEnum.down;
          } else {
            this.webServerStatus = this.ServerStatusEnum.unknown;
            console.log(err);
          }
        });
      }

      if (this.version === null) {
        this.api.get_version().then(response => {
          const build_sha_and_date = response.build.split("@");
          const last_build_date = (new Date(Date.parse(build_sha_and_date[1])))
          this.version = {
            version: response.version,
            build: response.build,
            build_sha_and_date: build_sha_and_date,
            full_version: `version: ${response.version} | build: ${response.build}`,
            readable_datetime: `${last_build_date} [${dayjs(last_build_date).fromNow()}]`
          }
        });
      }
    },
    toggleWireGuardNetworking() {
      const curr = this.wireguardStatus === this.ServerStatusEnum.up;
      this.api.post_wireguard_status({status: curr ? this.ServerStatusEnum.down : this.ServerStatusEnum.up})
          .then(() => {
            this.refresh();
          }).catch(err => {
        console.log(err);
      });
      this.wireguardStatus = this.ServerStatusEnum.unknown;
    },
    onPeerSelected(peer_id) {
      this.dialogId = `selected-peer-id=${peer_id}`;
    },
    logout() {
      this.api.token = '';
      localStorage.removeItem('token');
      localStorage.removeItem('remember');
      this.refresh();
    }
  }
}
</script>

