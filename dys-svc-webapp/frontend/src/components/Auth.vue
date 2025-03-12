<script setup lang="ts">
import {computed, inject, onMounted, ref} from "vue";
import type {VueCookies} from "vue-cookies";
import {CreateAccountRequest} from "%/services/auth/account.ts";
import {getAuthStore} from "@/stores/Auth.ts";
const $cookies = inject<VueCookies>('$cookies');
const authStore = getAuthStore();
const isLoggedIn = computed(() => authStore.cookie && (authStore.cookie.length > 0));

const createAccount = async () => {
  const accountName = (document.getElementById("auth-username") as HTMLInputElement).value;
  const request = CreateAccountRequest.fromJSON({
    accountName: accountName
  });

  const respStatus = (await fetch(`api/create_account`, {
    method: "POST",
    body: JSON.stringify(request)
  })).status;

  if (respStatus === 200) {
    $cookies?.set("dax-auth", accountName);
    authStore.cookie = accountName;
  }
};

const logout = async () => {
  $cookies?.remove('dax-auth');
  authStore.cookie = String();
};

onMounted(() => {
  authStore.cookie = $cookies?.get("dax-auth") || "";
})
</script>

<template>
  <div v-if="isLoggedIn">
    <p>Authenticated as {{authStore.cookie}}</p>
    <p @click="logout">Log Out</p>
  </div>
  <div v-else>
    <label for="username">Username (TEMP)</label>
    <input id="auth-username" name="username" type="text"/>
    <br>
    <button @click="createAccount">Create Account</button>
  </div>
</template>

<style scoped>
div {
  margin-left: auto;
}
</style>