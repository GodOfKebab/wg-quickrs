<template>

  <div class="text-sm text-gray-500 whitespace-pre grid grid-cols-2 gap-1">
    <div class="col-span-2 bg-blue-100 rounded-md overflow-scroll">
      <strong class="text-gray-600 justify-center rounded-md bg-blue-200 p-1">Changed fields</strong>
      <!--      <div v-if="Object.keys(peerEditChangedFieldsCompute[0]).length > 0 || !peerEditChangedFieldsCompute[3]">-->
      <!--        <div v-if="!peerEditChangedFieldsCompute[3]" class="p-1">-->
      <!--          {{ JSON.stringify(peerEditChangedFieldsCompute[0], false, 2) }}-->
      <!--        </div>-->
      <!--        <div-->
      <!--            v-else-if="('peers' in peerEditChangedFieldsCompute[0]) && !('connections' in peerEditChangedFieldsCompute[0])"-->
      <!--            class="p-1">{{ JSON.stringify({peers: peerEditChangedFieldsCompute[0].peers}, false, 2) }}-->
      <!--        </div>-->
      <!--        <div-->
      <!--            v-else-if="!('peers' in peerEditChangedFieldsCompute[0]) && ('connections' in peerEditChangedFieldsCompute[0])"-->
      <!--            class="p-1">{{ JSON.stringify({connections: peerEditChangedFieldsCompute[0].connections}, false, 2) }}-->
      <!--        </div>-->
      <!--        <div v-else class="p-1">{{ JSON.stringify(peerEditChangedFieldsCompute[0], false, 2) }}</div>-->
      <!--      </div>-->
    </div>
    <div class="col-span-2 bg-green-100 rounded-md overflow-scroll">
      <strong class="text-gray-600 justify-center rounded-md bg-green-200 p-1">Added fields</strong>
      <!--      <div v-if="Object.keys(peerEditChangedFieldsCompute[1]).length > 0" class="p-1">-->
      <!--        {{ JSON.stringify(peerEditChangedFieldsCompute[1], false, 2) }}-->
      <!--      </div>-->
    </div>
    <div class="col-span-2 bg-red-100 rounded-md overflow-scroll">
      <strong class="text-gray-600 justify-center rounded-md bg-red-200 p-1">Removed fields</strong>
      <!--      <div v-if="Object.keys(peerEditChangedFieldsCompute[2]).length > 0" class="p-1">-->
      <!--        {{ JSON.stringify(peerEditChangedFieldsCompute[2], false, 2) }}-->
      <!--      </div>-->
    </div>
    <div class="bg-red-100 rounded-md overflow-scroll">
      <strong class="text-gray-600 justify-center rounded-md bg-red-200 p-1">Old configuration</strong>
      <div v-if="error_detected" class="p-1">{}</div>
      <div v-else class="p-1">{{ JSON.stringify(network, false, 2) }}</div>
    </div>
    <div class="bg-green-100 rounded-md overflow-scroll">
      <strong class="text-gray-600 justify-center rounded-md bg-green-200 p-1">New configuration</strong>
      <!--      <div v-if="!peerEditChangedFieldsCompute[3]" class="p-1">{}</div>-->
      <!--      <div v-else class="p-1">{{ JSON.stringify(peerEditNewConfig, false, 2) }}</div>-->
    </div>
  </div>

</template>

<script>


export default {
  name: "change-sum",
  props: {
    changeSum: {
      type: Object,
      default: {
        changedFields: {},
        addedConnections: {},
        removedConnections: {}
      },
    },
    peerId: {
      type: String,
      default: "",
    },
    network: {
      type: Object,
      default: {},
    },
  },
  computed: {
    error_detected() {
      return false;
    },
  }
}
</script>

<style scoped>

</style>