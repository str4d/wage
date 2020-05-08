<template>
  <div>
    <font-awesome-icon icon="file" size="10x" />
    <p>TODO: decryption pane.</p>
    <div id="passphrase-section" v-if="needPassphrase">
      <label for="passphrase">Passphrase:</label>
      <input type="password" id="passphrase" />
    </div>
    <label class="button">Decrypt</label>
  </div>
</template>

<script>
export default {
  name: "DecryptPane",
  props: {
    needPassphrase: Boolean
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
