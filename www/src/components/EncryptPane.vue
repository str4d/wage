<template>
  <div>
    <font-awesome-icon icon="file" size="10x" />
    <p>TODO: encryption pane.</p>
    <div id="passphrase-section">
      <label for="passphrase">Passphrase:</label>
      <input type="password" id="passphrase" v-model="passphrase" />
    </div>
    <label class="button" @click="encryptFile">Encrypt with passphrase</label>
  </div>
</template>

<script>
export default {
  name: "EncryptPane",
  data() {
    return {
      passphrase: null,
    };
  },
  methods: {
    encryptFile() {
      this.$emit("encrypt-with-passphrase", this.passphrase);
    },
  },
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
