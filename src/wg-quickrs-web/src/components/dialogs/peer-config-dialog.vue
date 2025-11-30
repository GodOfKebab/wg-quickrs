<template>

  <div>
    <!-- Dialog: Settings -->
    <custom-dialog :left-button-click="() => { $emit('update:dialogId', ''); }"
                   :left-button-text="'Cancel'"
                   right-button-color="green"
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
        <span class="order-last w-full flex justify-between p-1 px-0 md:px-2 mb-1 mr-2">
          <delete-button :disabled="peerId === network.this_peer"
                         title="Delete this peer"
                         image-classes="h-10 w-10"
                         @click="overlayDialogId = 'confirm-delete'"></delete-button>
          <compare-button :active="page === 'view-changes'"
                          :disabled="!(changeDetected || errorDetected)"
                          image-classes="h-10 w-10"
                          title="See the configuration differences for this peer"
                          @click="page = 'view-changes'"></compare-button>
          <edit-button :active="page === 'edit'"
                       image-classes="h-10 w-10"
                       title="Edit the configuration for this peer"
                       @click="page = 'edit'"></edit-button>
          <conf-button :active="page === 'file'"
                       :disabled="changeDetected || errorDetected"
                       image-classes="h-10 w-10"
                       title="See the configuration file for this peer"
                       @click="page = 'file'"></conf-button>
          <qr-button :disabled="changeDetected || errorDetected"
                     image-classes="h-10 w-10"
                     title="Show QR Code"
                     @click="drawQRCode(); overlayDialogId = 'qr'"></qr-button>
          <download-button :disabled="changeDetected || errorDetected"
                           image-classes="h-10 w-10"
                           title="Download Configuration"
                           @click="downloadPeerConfig()"></download-button>
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
                               :network="network"
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
                          :is-this-peer="peerId === network.this_peer"
                          class="my-2 mr-2"
                          @updated-change-sum="onUpdatedScriptsIslandChangeSum"></scripts-island>

          <peer-details-island :api="api"
                               :peer="peer_conf"
                               class="my-2 mr-2"
                               @updated-change-sum="onUpdatedPeerDetailsIslandChangeSum"></peer-details-island>

          <peer-amnezia-params-island v-if="network.amnezia_parameters.enabled"
              :peer="peer_conf"
              class="my-2 mr-2"
              @updated-change-sum="onUpdatedAmneziaParametersIslandChangeSum"></peer-amnezia-params-island>

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
                   right-button-color="green"
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
                   right-button-color="red"
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
import CustomDialog from "@/src/components/dialogs/custom-dialog.vue";
import PeerSummaryIsland from "@/src/components/islands/peer-summary.vue";
import PeerKindIconIsland from "@/src/components/islands/peer-kind-icon.vue";
import DNSMTUIsland from "@/src/components/islands/dnsmtu.vue";
import ScriptsIsland from "@/src/components/islands/scripts.vue";
import PeerDetails from "@/src/components/islands/peer-details.vue";
import ConnectionIslands from "@/src/components/islands/connections.vue";
import ChangeSum from "@/src/components/change-sum.vue";
import DeleteButton from "@/src/components/ui/buttons/delete.vue";
import QRCode from "qrcode";
import CompareButton from "@/src/components/ui/buttons/compare.vue";
import EditButton from "@/src/components/ui/buttons/edit.vue";
import ConfButton from "@/src/components/ui/buttons/conf.vue";
import QrButton from "@/src/components/ui/buttons/qr.vue";
import DownloadButton from "@/src/components/ui/buttons/download.vue";
import {get_peer_wg_config_wasm} from "@/pkg/wg_quickrs_lib.js";
import PeerAmneziaParamsIsland from "@/src/components/islands/peer-amnezia-params.vue";

export default {
  name: "peer-config-dialog",
  components: {
    PeerAmneziaParamsIsland,
    DownloadButton,
    QrButton,
    ConfButton,
    EditButton,
    CompareButton,
    'custom-dialog': CustomDialog,
    'peer-summary-island': PeerSummaryIsland,
    'peer-kind-icon-island': PeerKindIconIsland,
    'dnsmtu-island': DNSMTUIsland,
    'scripts-island': ScriptsIsland,
    'peer-details-island': PeerDetails,
    'connection-islands': ConnectionIslands,
    'change-sum': ChangeSum,
    DeleteButton
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
      amnesiaParametersIslandChangeSum: null,
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
    onUpdatedAmneziaParametersIslandChangeSum(data) {
      this.amnesiaParametersIslandChangeSum = data;
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
      const peerConfigFileContents = get_peer_wg_config_wasm(this.network, this.peerId);
      const peerConfigFileName = this.network.peers[this.peerId].name.replace(/[^a-zA-Z0-9_=+.-]/g, '-').replace(/(-{2,}|-$)/g, '-').replace(/-$/, '').substring(0, 32);

      const element = document.createElement('a');
      element.setAttribute('href', `data:text/plain;charset=utf-8,${encodeURIComponent(peerConfigFileContents)}`);
      element.setAttribute('download', `${this.network.name}-${peerConfigFileName}.conf`);

      element.style.display = 'none';
      document.body.appendChild(element);

      element.click();

      document.body.removeChild(element);

    }
  },
  computed: {
    peer_conf() {
      return this.network.peers[this.peerId];
    },
    peer_wg_conf_file() {
      return get_peer_wg_config_wasm(this.network, this.peerId);
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
      for (const island_datum of [this.peerSummaryIslandChangeSum, this.peerKindIconIslandChangeSum, this.dnsmtuIslandChangeSum, this.scriptsIslandChangeSum, this.peerDetailsIslandChangeSum, this.amnesiaParametersIslandChangeSum]) {
        if (!island_datum) continue;
        for (const [island_field, island_value] of Object.entries(island_datum.errors)) {
          if (island_field === "scripts" && island_value) {
            data.errors.peers[this.peerId].scripts = {};
            for (const [script_field, script_value] of Object.entries(island_value)) {
              if (script_field) data.errors.peers[this.peerId].scripts[script_field] = script_value;
            }
            if (Object.keys(data.errors.peers[this.peerId].scripts).length === 0) delete data.errors.peers[this.peerId].scripts;
          } else if (island_field === "amnezia_parameters" && island_value) {
            data.errors.peers[this.peerId].amnezia_parameters = {};
            for (const [amnezia_field, amnezia_value] of Object.entries(island_value)) {
              if (amnezia_value) data.errors.peers[this.peerId].amnezia_parameters[amnezia_field] = amnezia_value;
            }
            if (Object.keys(data.errors.peers[this.peerId].amnezia_parameters).length === 0) delete data.errors.peers[this.peerId].amnezia_parameters;
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
          } else if (island_field === "amnezia_parameters" && island_value) {
            data.changed_fields.peers[this.peerId].amnezia_parameters = {};
            for (const [amnezia_field, amnezia_value] of Object.entries(island_value)) {
              if (amnezia_value) data.changed_fields.peers[this.peerId].amnezia_parameters[amnezia_field] = amnezia_value;
            }
            if (Object.keys(data.changed_fields.peers[this.peerId].amnezia_parameters).length === 0) delete data.changed_fields.peers[this.peerId].amnezia_parameters;
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
      return !!(Object.keys(this.changeSum.changed_fields.peers[this.peerId]).length
          + Object.keys(this.changeSum.changed_fields.connections).length
          + Object.keys(this.changeSum.added_connections).length
          + Object.keys(this.changeSum.removed_connections).length);
    }
  },
}
</script>

<style scoped>

</style>