<template>

  <div>
    <!-- Dialog: Settings -->
    <custom-dialog :left-button-click="() => { $emit('update:dialogId', ''); }"
                   :left-button-text="'Cancel'"
                   :right-button-classes="['enabled:bg-green-700', 'enabled:hover:bg-green-800', 'enabled:border-green-900', 'enabled:focus:outline-none', 'bg-gray-200', 'disabled:hover:bg-gray-200', 'disabled:cursor-not-allowed', 'disabled:border-gray-200', 'text-white']"
                   :right-button-click="page === 'file' ? () => { navigator.clipboard.writeText(peer_wg_conf_file).then(() => {
                                          alert('successfully copied');
                                          })
                                          .catch(() => {
                                          alert('something went wrong');
                                          }); } : () => { overlayDialogId = 'confirm-changes' }"
                   :right-button-text="page === 'file' ? 'Copy To Clipboard' : 'Save Configuration'"
                   :rightButtonDisabled="page !== 'file' && (!changeDetected || errorDetected)"
                   class="z-10">

      <!-- title and top bar -->
      <div class="flex flex-col items-center">
        <h3 class="text-3xl leading-tight font-medium text-gray-900 inline mb-5 text-start w-full">
          Configuration for <strong>{{ peer_conf.name }}</strong>:
        </h3>
        <span class="order-last w-full flex justify-between p-1 px-2 mb-1 pr-4">
          <button :disabled="peerId === network.this_peer"
                  class="align-middle bg-gray-100 disabled:opacity-40 hover:enabled:bg-gray-600 p-1 px-2 rounded transition special-fill enabled:cursor-pointer"
                  title="Delete this peer"
                  @click="overlayDialogId = 'confirm-delete'">
            <img alt="Delete" class="h-10" src="/icons/flowbite/trash-bin.svg"/>
          </button>
          <button :disabled="!changeDetected"
                  class="align-middle bg-gray-100 disabled:opacity-40 hover:enabled:bg-gray-600 p-1 px-2 rounded transition special-fill enabled:cursor-pointer"
                  :class="page === 'view-changes' ? 'bg-gray-600' : ''"
                  title="See the configuration differences for this peer"
                  @click="page = 'view-changes'">
            <img alt="Compare Configuration" class="h-10" src="/icons/flowbite/merge-cells.svg"/>
          </button>
          <button
              class="align-middle bg-gray-100 hover:bg-gray-600 p-1 px-2 rounded transition special-fill-edit enabled:cursor-pointer"
                  :class="page === 'edit' ? 'bg-gray-600' : ''"
                  title="Edit the configuration for this peer"
                  @click="page = 'edit'">
            <img alt="Edit Configuration" class="h-10" src="/icons/flowbite/file-pen.svg"/>
          </button>
          <button :disabled="changeDetected"
                  class="align-middle bg-gray-100 disabled:opacity-40 hover:enabled:bg-gray-600 p-1 px-2 rounded transition special-fill enabled:cursor-pointer"
                  :class="page === 'file' ? 'bg-gray-600' : ''"
                  title="See the configuration file for this peer"
                  @click="page = 'file'">
            <img alt="WireGuard Configuration File" class="h-10" src="/icons/flowbite/file-code.svg"/>
          </button>
          <button :disabled="changeDetected"
                  class="align-middle bg-gray-100 disabled:opacity-40 hover:enabled:bg-gray-600 p-1 px-2 rounded transition special-fill enabled:cursor-pointer"
                  :class="overlayDialogId === 'qr' ? 'bg-gray-600' : ''"
                  title="Show QR Code"
                  @click="drawQRCode(); overlayDialogId = 'qr'">
            <img alt="QR Code" class="h-10" src="/icons/flowbite/qr-code.svg"/>
          </button>
          <button :disabled="changeDetected"
                  class="align-middle bg-gray-100 disabled:opacity-40 hover:enabled:bg-gray-600 p-1 px-2 rounded transition special-fill enabled:cursor-pointer"
                  title="Download Configuration"
                  @click="downloadPeerConfig()">
            <img alt="Download" class="h-10" src="/icons/flowbite/download.svg"/>
          </button>
        </span>
      </div>

      <div class="flex max-h-[calc(100vh-20rem)] flex-col overflow-y-auto">
        <!-- show config -->
        <div v-show="page === 'file'" class="mt-2 w-full overflow-scroll text-start">
          <span class="text-sm text-gray-500 whitespace-pre">{{ peer_wg_conf_file }}</span>
        </div>

        <!-- edit config -->
        <div v-show="page === 'edit'" class="mt-0 w-full overflow-scroll text-start">

          <peer-summary-island :is-host="peerId === network.this_peer"
                               :peer="peer_conf"
                               class="my-2 mr-2"
                               @updated-change-sum="onUpdatedPeerSummaryIslandChangeSum"></peer-summary-island>

          <peer-kind-icon-island
              :default-kind-icon="{kind: network.defaults.peer.kind, icon: network.defaults.peer.icon}"
              :peer="peer_conf"
              class="my-2 mr-2"
              @updated-change-sum="onUpdatedPeerKindIconIslandChangeSum"></peer-kind-icon-island>

          <dnsmtu-island :default-dnsmtu="{dns: network.defaults.peer.dns, mtu: network.defaults.peer.mtu}"
                         :peer="peer_conf"
                         class="my-2 mr-2"
                         @updated-change-sum="onUpdatedDnsmtuIslandChangeSum"></dnsmtu-island>

          <scripts-island :default-scripts="network.defaults.peer.scripts"
                          :peer="peer_conf"
                          class="my-2 mr-2"
                          @updated-change-sum="onUpdatedScriptsIslandChangeSum"></scripts-island>

          <peer-details-island :api="api"
                               :peer="peer_conf"
                               class="my-2 mr-2"
                               @updated-change-sum="onUpdatedPeerDetailsIslandChangeSum"></peer-details-island>

          <connection-islands :api="api"
                              :network="network"
                              :peer-id="peerId"
                              class="my-2 mr-2"
                              @updated-change-sum="onUpdatedConnectionsIslandsChangeSum"></connection-islands>
        </div>

        <!-- view changes -->
        <div v-show="page === 'view-changes'" class="mt-2 w-full overflow-scroll text-start">
          <change-sum :change-sum="changeSum"
                      :network="network"
                      :peer-id="peerId"></change-sum>
        </div>
      </div>

    </custom-dialog>

    <!-- Dialog: Confirm Changes-->
    <custom-dialog v-if="overlayDialogId === 'confirm-changes'"
                   :left-button-click="() => { overlayDialogId = ''; }"
                   :left-button-text="'Cancel'"
                   :right-button-classes="['text-white', 'bg-green-600', 'hover:bg-green-700']"
                   :right-button-click="() => { updateConfiguration(); overlayDialogId = ''; page = 'edit'; $emit('update:dialogId', ''); }"
                   :right-button-text="'Do it!'"
                   class="z-20"
                   icon="danger">
      <h3 class="text-lg leading-6 font-medium text-gray-900">
        Confirm changes for <strong>{{ peer_conf.name }}</strong>
      </h3>
      <div class="my-2 text-sm text-gray-500">
        Are you sure you want to make these changes?
      </div>

      <div class="flex max-h-[calc(100vh-20rem)] flex-col overflow-y-auto">
        <change-sum :change-sum="changeSum"
                    :network="network"
                    :peer-id="peerId"></change-sum>
      </div>
    </custom-dialog>

    <!-- Dialog: Confirm Delete -->
    <custom-dialog v-if="overlayDialogId === 'confirm-delete'"
                   :left-button-click="() => { overlayDialogId = '' }"
                   :left-button-text="'Cancel'"
                   :right-button-classes="['text-white', 'bg-red-600', 'hover:bg-red-700']"
                   :right-button-click="() => { deletePeer(); overlayDialogId = ''; page = 'edit'; $emit('update:dialogId', ''); }"
                   :right-button-text="'Delete!'"
                   class="z-20"
                   icon="danger">
      <h3 class="text-lg leading-6 font-medium text-gray-900">
        Confirm deleting <strong>{{ peer_conf.name }}</strong>
      </h3>
      <div class="my-2 text-sm text-gray-500">
        Are you sure you want to delete this peer?
      </div>
    </custom-dialog>

    <!-- Window: QR Code Display -->
    <div v-show="overlayDialogId === 'qr'">
      <div class="bg-black bg-opacity-50 fixed top-0 right-0 left-0 bottom-0 flex items-center justify-center z-20">
        <div class="bg-white rounded-md shadow-lg relative p-8">
          <button class="absolute right-4 top-4 text-gray-600 hover:text-gray-800" @click="overlayDialogId = ''">
            <img alt="Close QR Code Window" class="h-6" src="/icons/flowbite/close.svg"/>
          </button>
          <canvas id="qr-canvas"></canvas>
        </div>
      </div>
    </div>

  </div>

</template>

<script>
import CustomDialog from "@/components/dialogs/custom-dialog.vue";
import PeerSummaryIsland from "@/components/islands/peer-summary.vue";
import PeerKindIconIsland from "@/components/islands/peer-kind-icon.vue";
import DNSMTUIsland from "@/components/islands/dnsmtu.vue";
import ScriptsIsland from "@/components/islands/scripts.vue";
import PeerDetails from "@/components/islands/peer-details.vue";
import ConnectionIslands from "@/components/islands/connections.vue";
import ChangeSum from "@/components/change-sum.vue";
import WireGuardHelper from "@/js/wg-helper.js";
import QRCode from "qrcode";

export default {
  name: "peer-config-dialog",
  components: {
    'custom-dialog': CustomDialog,
    'peer-summary-island': PeerSummaryIsland,
    'peer-kind-icon-island': PeerKindIconIsland,
    'dnsmtu-island': DNSMTUIsland,
    'scripts-island': ScriptsIsland,
    'peer-details-island': PeerDetails,
    'connection-islands': ConnectionIslands,
    'change-sum': ChangeSum,
  },
  props: {
    peerId: {
      type: String,
      default: "",
    },
    network: {
      type: Object,
      default: {},
    },
    dialogId: {
      type: String,
      default: "",
    },
    version: {
      type: Object,
      default: {},
    },
    api: {
      type: Object,
      default: null,
    }
  },
  emits: ['update:dialogId'],
  data() {
    return {
      page: "",

      peerSummaryIslandChangeSum: null,
      peerKindIconIslandChangeSum: null,
      dnsmtuIslandChangeSum: null,
      scriptsIslandChangeSum: null,
      peerDetailsIslandChangeSum: null,
      connectionIslandsChangeSum: {
        changed_fields: {},
        added_connections: {},
        removed_connections: {},
        errors: {},
      },
      overlayDialogId: '',
    }
  },
  mounted: function () {
    this.page = 'edit'
  },
  methods: {
    onUpdatedPeerSummaryIslandChangeSum(data) {
      this.peerSummaryIslandChangeSum = data;
    },
    onUpdatedPeerKindIconIslandChangeSum(data) {
      this.peerKindIconIslandChangeSum = data;
    },
    onUpdatedDnsmtuIslandChangeSum(data) {
      this.dnsmtuIslandChangeSum = data;
    },
    onUpdatedScriptsIslandChangeSum(data) {
      this.scriptsIslandChangeSum = data;
    },
    onUpdatedPeerDetailsIslandChangeSum(data) {
      this.peerDetailsIslandChangeSum = data;
    },
    onUpdatedConnectionsIslandsChangeSum(data) {
      this.connectionIslandsChangeSum = data;
    },
    updateConfiguration() {
      this.api.patch_network_config({
        changed_fields: this.changeSum.changed_fields,
        added_connections: this.changeSum.added_connections,
        removed_connections: Object.keys(this.changeSum.removed_connections)
      });
    },
    deletePeer() {
      const changeSum = {
        removed_peers: [this.peerId],
        removed_connections: Object.keys(this.network.connections).filter(id => id.includes(this.peerId))
      };
      this.api.patch_network_config(changeSum);
    },
    drawQRCode() {
      QRCode.toCanvas(document.getElementById('qr-canvas'), this.peer_wg_conf_file);
    },
    downloadPeerConfig() {
      WireGuardHelper.downloadPeerConfig(this.network, this.peerId, this.version);
    }
  },
  computed: {
    peer_conf() {
      return this.network.peers[this.peerId];
    },
    peer_wg_conf_file() {
      return WireGuardHelper.getPeerConfig(this.network, this.peerId, this.version.full_version);
    },
    changeSum() {
      const data = {
        errors: {
          peers: {},
          connections: {},
        },
        changed_fields: {
          peers: {},
          connections: {},
        },
        added_connections: {},
        removed_connections: {}
      };

      // check for errors + changed fields for this peer
      data.errors.peers[this.peerId] = {};
      data.changed_fields.peers[this.peerId] = {};
      for (const island_datum of [this.peerSummaryIslandChangeSum, this.peerKindIconIslandChangeSum, this.dnsmtuIslandChangeSum, this.scriptsIslandChangeSum, this.peerDetailsIslandChangeSum]) {
        if (!island_datum) continue;
        for (const [island_field, island_value] of Object.entries(island_datum.errors)) {
          if (island_field === "scripts" && island_value) {
            data.errors.peers[this.peerId].scripts = {};
            for (const [script_field, script_value] of Object.entries(island_value)) {
              if (script_field) data.errors.peers[this.peerId].scripts[script_field] = script_value;
            }
            if (Object.keys(data.errors.peers[this.peerId].scripts).length === 0) delete data.errors.peers[this.peerId].scripts;
          } else {
            if (island_value) data.errors.peers[this.peerId][island_field] = island_value;
          }
        }

        for (const [island_field, island_value] of Object.entries(island_datum.changed_fields)) {
          if (island_field === "scripts" && island_value) {
            data.changed_fields.peers[this.peerId].scripts = {};
            for (const [script_field, script_value] of Object.entries(island_value)) {
              if (script_field) data.changed_fields.peers[this.peerId].scripts[script_field] = script_value;
            }
            if (Object.keys(data.changed_fields.peers[this.peerId].scripts).length === 0) delete data.changed_fields.peers[this.peerId].scripts;
          } else {
            if (island_value) data.changed_fields.peers[this.peerId][island_field] = island_value;
          }
        }
      }

      data.changed_fields.connections = this.connectionIslandsChangeSum.changed_fields
      data.added_connections = this.connectionIslandsChangeSum.added_connections;
      data.removed_connections = this.connectionIslandsChangeSum.removed_connections;
      data.errors.connections = this.connectionIslandsChangeSum.errors;

      return data;
    },
    errorDetected() {
      return !!(Object.keys(this.changeSum.errors.peers[this.peerId]).length
          + Object.keys(this.changeSum.errors.connections).length);
    },
    changeDetected() {
      return !!(this.errorDetected
          + Object.keys(this.changeSum.changed_fields.peers[this.peerId]).length
          + Object.keys(this.changeSum.changed_fields.connections).length
          + Object.keys(this.changeSum.added_connections).length
          + Object.keys(this.changeSum.removed_connections).length);
    }
  },
}
</script>

<style scoped>

</style>