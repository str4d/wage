<template>
  <section class="box">
    <b-field label="Passphrase">
      <b-input v-model="passphrase" type="password" password-reveal> </b-input>
    </b-field>
    <b-button @click="encryptFile">Encrypt</b-button>
  </section>
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
