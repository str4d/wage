<template>
  <div>
    <font-awesome-icon icon="file" size="10x" />
    <p>TODO: decryption pane.</p>
    <div id="passphrase-section" v-if="!fileDecrypted && needPassphrase">
      <label for="passphrase">Passphrase:</label>
      <input type="password" id="passphrase" v-model="passphrase" />
    </div>
    <label class="button" v-if="!fileDecrypted" @click="decryptFile">Decrypt</label>
    <label class="button" v-if="fileDecrypted" @click="downloadFile">Download</label>
  </div>
</template>

<script>
export default {
  name: "DecryptPane",
  props: {
    fileDecrypted: Boolean,
    needPassphrase: Boolean
  },
  data() {
    return {
      passphrase: null
    };
  },
  methods: {
    decryptFile() {
      this.$emit("decrypt-with-passphrase", this.passphrase);
    },
    downloadFile() {
      this.$emit("download-file");
    }
  }
};
</script>

<style scoped>
#passphrase-section {
  margin-bottom: 10px;
}
#passphrase {
  margin-left: 10px;
}
</style>
