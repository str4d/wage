<template>
  <section class="box">
    <div v-if="needIdentities">
      <p>Requires identities to decrypt, which are not supported yet.</p>
      <p>Check back soon!</p>
    </div>
    <b-field label="Passphrase" v-if="!fileDecrypted && needPassphrase">
      <b-input type="password" v-model="passphrase" password-reveal> </b-input>
    </b-field>
    <b-button
      v-if="!fileDecrypted"
      v-bind:disabled="decryptDisabled"
      @click="decryptFile"
      >Decrypt</b-button
    >
    <b-button v-if="fileDecrypted" @click="downloadFile">Download</b-button>
  </section>
</template>

<script>
export default {
  name: "DecryptPane",
  props: {
    fileDecrypted: Boolean,
    needIdentities: Boolean,
    needPassphrase: Boolean,
  },
  data() {
    return {
      passphrase: null,
    };
  },
  computed: {
    // Button disabling
    decryptDisabled() {
      return !(
        (this.needIdentities && false) ||
        (this.needPassphrase && this.passphrase.length)
      );
    },
  },
  methods: {
    decryptFile() {
      if (this.needIdentities) {
        console.log("TODO: Decrypt with identities");
      } else if (this.needPassphrase) {
        this.$emit("decrypt-with-passphrase", this.passphrase);
      }
    },
    downloadFile() {
      this.$emit("download-file");
    },
  },
};
</script>
