<template>
  <section class="columns">
    <div class="column is-narrow">
      <font-awesome-icon :icon="fileIcon" :size="iconSize" />
    </div>
    <div id="file-info" class="column">
      <button id="reset-app" class="delete is-large" @click="resetApp()" />
      <slot></slot>
    </div>
  </section>
</template>

<script>
export default {
  name: "FileInfo",
  props: ["fileIcon", "iconSize"],
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
