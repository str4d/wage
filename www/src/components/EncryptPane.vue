<template>
  <div>
    <font-awesome-icon icon="file" size="10x" />
    <p>TODO: encryption pane.</p>
    <b-field label="Passphrase">
      <b-input v-model="passphrase" type="password" password-reveal> </b-input>
    </b-field>
    <b-button @click="encryptFile">Encrypt</b-button>
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
