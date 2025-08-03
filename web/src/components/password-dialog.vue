<template>
  <div id="authentication-modal" aria-hidden="true"
       class="flex overflow-y-auto overflow-x-hidden fixed top-0 right-0 left-0 z-50 justify-center items-center w-full md:inset-0 h-[calc(100%-1rem)] max-h-full"
       tabindex="-1">
    <div class="relative p-4 w-full max-w-md max-h-full">
      <div class="fixed inset-0 bg-black bg-opacity-50 z-40"></div>

      <!-- Modal content -->
      <div class="relative bg-gray-100 rounded-lg shadow-sm dark:bg-gray-700 z-50">
        <!-- Modal header -->
        <div
            class="flex items-center justify-between p-4 md:p-5 border-b rounded-t dark:border-gray-600 border-gray-200">
          <h3 class="text-xl font-semibold text-gray-900 dark:text-white text-center w-full">
            Sign in to wg-rusteze
          </h3>
        </div>
        <!-- Modal body -->
        <div class="p-4 md:p-5">
          <form action="#" class="space-y-4"
                @submit.prevent="on_submit()">
            <div>
              <label
                  :class="wrong_password ? ['text-red-700', 'dark:text-red-500'] : ['text-gray-900', 'dark:text-white']"
                  class="block mb-2 text-sm font-medium"
                  for="password">Password</label>
              <input id="password"
                     v-model="password"
                     :class="wrong_password ?
                     ['bg-red-50',  'border-red-500',  'text-red-900',  'placeholder-red-700', 'focus:ring-red-500', 'focus:border-red-500', 'dark:text-red-500', 'dark:placeholder-red-500', 'dark:placeholder-red-500', 'dark:border-red-500', 'text-red-600', 'dark:text-red-500'] :
                     ['bg-gray-50', 'border-gray-300', 'text-gray-900', 'focus:ring-blue-500', 'focus:border-blue-500', 'dark:bg-gray-600', 'dark:border-gray-500', 'dark:placeholder-gray-400', 'dark:text-white']"
                     class=" border text-sm rounded-lg block w-full p-2.5"
                     name="password"
                     placeholder="••••••••"
                     required
                     type="password"/>
              <p v-if="wrong_password" class="mt-2 text-sm"><span class="font-medium">Oops!</span> Incorrect Password!
              </p>
            </div>

            <div class="flex justify-between">
              <div class="flex items-start">
                <div class="flex items-center h-5">
                  <input id="remember"
                         v-model="remember"
                         class="w-4 h-4 border border-gray-300 rounded-sm bg-gray-50 focus:ring-3 focus:ring-blue-300 dark:bg-gray-600 dark:border-gray-500 dark:focus:ring-blue-600 dark:ring-offset-gray-800 dark:focus:ring-offset-gray-800"
                         type="checkbox"/>
                </div>
                <label class="ms-2 text-sm font-medium text-gray-900 dark:text-gray-300" for="remember">Remember
                  me</label>
              </div>
              <!--              <a class="text-sm text-blue-700 hover:underline dark:text-blue-500" href="#">Lost Password?</a>-->
            </div>
            <button
                class="w-full text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800"
                type="submit">
              Login to your account
            </button>
          </form>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
export default {
  name: "password-dialog",
  props: {
    api: {
      type: Object,
      default: null,
    }
  },
  data() {
    return {
      password: "",
      remember: true,
      wrong_password: false,
    }
  },
  methods: {
    async on_submit() {
      this.api.refresh_api_token(this.password).then((_) => {
        if (this.remember) {
          localStorage.setItem('token', this.api.token);
          localStorage.setItem('remember', 'true');
        } else {
          localStorage.removeItem('token');
          localStorage.setItem('remember', 'false');
        }
      }).catch((_) => {
        this.wrong_password = true;
      });
    }
  }
}
</script>

<style scoped>
</style>