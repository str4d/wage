<template>
  <div>
    <font-awesome-icon :icon="fileIcon" size="10x" />
    <p>TODO: decryption pane.</p>
    <b-field label="Passphrase" v-if="!fileDecrypted && needPassphrase">
      <b-input type="password" v-model="passphrase" password-reveal> </b-input>
    </b-field>
    <b-button v-if="!fileDecrypted" @click="decryptFile">Decrypt</b-button>
    <b-button v-if="fileDecrypted" @click="downloadFile">Download</b-button>
  </div>
</template>

<script>
export default {
  name: "DecryptPane",
  props: {
    fileIcon: String,
    fileDecrypted: Boolean,
    needPassphrase: Boolean,
  },
  data() {
    return {
      passphrase: null,
    };
  },
  methods: {
    decryptFile() {
      this.$emit("decrypt-with-passphrase", this.passphrase);
    },
    downloadFile() {
      this.$emit("download-file");
    },
  },
};
</script>
