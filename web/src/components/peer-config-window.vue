<template>

  <div>
    <custom-dialog :left-button-click="() => { peerId = null }"
                   :left-button-text="'Cancel'"
                   :right-button-classes="['enabled:bg-green-700', 'enabled:hover:bg-green-800', 'enabled:focus:outline-none', 'bg-gray-200', 'disabled:hover:bg-gray-200', 'disabled:cursor-not-allowed', 'text-white']"
                   :right-button-click="peerConfigWindow === 'file' ? () => { navigator.clipboard.writeText(peer_wg_conf_file).then(() => {
                                          alert('successfully copied');
                                          })
                                          .catch(() => {
                                          alert('something went wrong');
                                          }); } : () => { dialogId = 'confirm-changes'; }"
                   :right-button-text="peerConfigWindow === 'file' ? 'Copy To Clipboard' : 'Save Configuration'"
                   class="z-10">

      <div class="flex justify-between items-center">
        <h3 class="text-lg leading-6 font-medium text-gray-900 inline">
          Configuration for <strong>{{ peer_conf.name }}</strong>:
        </h3>
        <span class="order-last">
          <button v-show="peerConfigWindow === 'edit'"
                  class="align-middle bg-gray-100 hover:enabled:bg-gray-600 hover:enabled:text-white disabled:text-blue-100 disabled:bg-gray-50 p-1 rounded transition special-fill mr-1"
                  title="See the configuration differences for this peer"
                  @click="peerConfigWindow = 'view-changes'">
            <img alt="Compare Configuration" class="h-6" src="../icons/flowbite/file-clone.svg"/>
          </button>
          <button v-show="peerConfigWindow === 'file' || peerConfigWindow === 'view-changes'"
                  class="align-middle bg-gray-100 hover:bg-gray-600 hover:text-white p-1 rounded transition special-fill-edit"
                  title="Edit the configuration for this peer" @click="peerConfigWindow = 'edit'">
            <img alt="Edit Configuration" class="h-6" src="../icons/flowbite/file-pen.svg"/>
          </button>
          <button v-show="peerConfigWindow === 'edit'"
                  class="align-middle bg-gray-100 hover:bg-gray-600 hover:text-white p-1 rounded transition special-fill"
                  title="See the configuration file for this peer" @click="peerConfigWindow = 'file'">
            <img alt="WireGuard Configuration File" class="h-6" src="../icons/flowbite/file-code.svg"/>
          </button>
        </span>
      </div>

      <!-- show config -->
      <div v-show="peerConfigWindow === 'file'" class="mt-2 w-full overflow-scroll h-96">
        <span class="text-sm text-gray-500 whitespace-pre">{{ peer_wg_conf_file }}</span>
      </div>

      <!-- edit config -->
      <div v-show="peerConfigWindow === 'edit'" class="mt-2 w-full overflow-auto h-96">

        <peer-summary-island v-model="peerSummaryIslandData"
                             :peer="peer_conf"
                             class="my-2 mr-2"></peer-summary-island>

        <dnsmtu-island v-model="dnsmtuIslandData"
                       :default-dnsmtu="{dns: network.defaults.peer.dns, mtu: network.defaults.peer.mtu}"
                       :dnsmtu="{dns: peer_conf.dns, mtu: peer_conf.mtu}"
                       class="my-2 mr-2"></dnsmtu-island>

        <scripts-island v-model="scriptsIslandData"
                        :default-scripts="network.defaults.peer.scripts"
                        :scripts="peer_conf.scripts"
                        class="my-2 mr-2"></scripts-island>

        <peer-details-island v-model="peerDetailsIslandData"
                             :peer="peer_conf"
                             class="my-2 mr-2"></peer-details-island>

        <connection-islands v-model="connectionIslandsData"
                            :network="network"
                            :peer-id="peerId"
                            class="my-2 mr-2"></connection-islands>
      </div>

      <!-- show config -->
      <div v-show="peerConfigWindow === 'view-changes'" class="mt-2 w-full overflow-scroll h-96">
        <change-sum :change-sum="change_sum"
                    :network="network"
                    :peer-id="peerId"></change-sum>
      </div>
    </custom-dialog>

    <!-- Dialog: Confirm -->
    <custom-dialog v-if="dialogId === 'confirm-changes'" :left-button-click="() => { dialogId = null }"
                   :left-button-text="'Cancel'"
                   :right-button-classes="['text-white', 'bg-green-600', 'hover:bg-green-700']"
                   :right-button-click="() => { peerId = null; peerConfigWindow = 'edit'; dialogId = null; }"
                   :right-button-text="'Do it!'"
                   class="z-20"
                   icon="danger">
      <h3 class="text-lg leading-6 font-medium text-gray-900">
        Confirm changes for <strong>{{ peer_conf.name }}</strong>
      </h3>
      <div class="mt-2 text-sm text-gray-500">
        Are you sure you want to make these changes?
      </div>

      <!--      <change-sum :peer-edit-changed-fields-compute="peerEditChangedFieldsCompute"-->
      <!--                  :peer-edit-new-config="peerEditNewConfig"-->
      <!--                  :peer-edit-old-config="peerEditOldConfig"></change-sum>-->
    </custom-dialog>
  </div>

</template>

<script>
import CustomDialog from "./custom-dialog.vue";
import PeerSummaryIsland from "./islands/peer-summary.vue";
import DNSMTUIsland from "./islands/dnsmtu.vue";
import ScriptsIsland from "./islands/scripts.vue";
import PeerDetails from "./islands/peer-details.vue";
import ConnectionIslands from "./islands/connections.vue";
import ChangeSum from "./change-sum.vue";
import WireGuardHelper from "../js/wg-helper";


export default {
  name: "peer-config-window",
  components: {
    'custom-dialog': CustomDialog,
    'peer-summary-island': PeerSummaryIsland,
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
  },
  data() {
    return {
      dialogId: '',

      peerConfigWindow: "",

      peerSummaryIslandData: {
        context: 'edit',
        changedFields: {},
        error: null,
      },
      dnsmtuIslandData: {
        context: 'edit',
        changedFields: {},
        error: null,
      },
      scriptsIslandData: {
        context: 'edit',
        changedFields: {},
        error: null,
      },
      peerDetailsIslandData: {
        context: 'edit',
        changedFields: {},
        error: null,
      },
      connectionIslandsData: {
        context: 'edit',
        addedConnections: {},
        removedConnections: {},
        changedFields: {},
        error: null,
      },

      change_sum: null,
    }
  },
  watch: {
    peerSummaryIslandData: {
      deep: true,
      handle() {
        console.log("anan")
        this.update_change_sum();
      },
    },
    dnsmtuIslandData: {
      handle(newValue, oldValue) {
        this.update_change_sum();
      },
      deep: true
    },
    scriptsIslandData: {
      handle(newValue, oldValue) {
        this.update_change_sum();
      },
      deep: true
    },
    peerDetailsIslandData: {
      handle(newValue, oldValue) {
        this.update_change_sum();
      },
      deep: true
    },
  },
  mounted: function () {
    this.peerConfigWindow = 'edit'
  },
  methods: {
    update_change_sum() {
      const data = {
        changedFields: {
          peers: {},
          connections: {},
        },
        addedConnections: {},
        removedConnections: {}
      };
      for (const island_datum of [this.peerSummaryIslandData,
        this.dnsmtuIslandData,
        this.scriptsIslandData,
        this.peerDetailsIslandData]) {
        if (island_datum.error) continue;

        for (const [island_field, island_value] of Object.entries(island_datum.changedFields)) {
          data.changedFields.peers[island_field] = island_value;
        }
      }
      if (Object.keys(data.changedFields.peers).length === 0) delete data.changedFields.peers;

      this.change_sum = data;
      console.log(this.change_sum)
    }
  },
  computed: {
    peer_conf() {
      return this.network.peers[this.peerId];
    },
    peer_wg_conf_file() {
      return WireGuardHelper.getPeerConfig(this.network, this.peerId);
    },
    change_sum() {
    }
  },
}
</script>

<style scoped>

</style>