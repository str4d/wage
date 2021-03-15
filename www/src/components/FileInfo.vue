<template>
  <section class="columns">
    <div class="column is-narrow">
      <font-awesome-icon :icon="fileIcon" size="10x" />
    </div>
    <div id="file-info" class="column">
      <button id="reset-app" class="delete is-large" @click="resetApp()" />
      <p>TODO: file info.</p>
    </div>
  </section>
</template>

<script>
export default {
  name: "FileInfo",
  props: ["fileIcon"],
  methods: {
    resetApp() {
      this.$emit("reset-app");
    },
  },
};
</script>

<style scoped>
#file-info {
  text-align: left;
}
#reset-app {
  float: right;
}
</style>
