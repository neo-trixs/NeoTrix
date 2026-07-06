#include <dlfcn.h>
#include <unistd.h>
#include <string.h>
#include <stdlib.h>
#include <stdio.h>
#include <time.h>
#include <sys/types.h>
#include <sys/sysctl.h>
#include <sys/utsname.h>
#include <pwd.h>

/* real function pointers — __sysctl/__sysctlbyname are in libsystem_kernel.dylib */
static int (*real___sysctl)(int *, u_int, void *, size_t *, void *, size_t) = NULL;
static int (*real___sysctlbyname)(const char *, void *, size_t *, void *, size_t) = NULL;
static int (*real_uname)(struct utsname *) = NULL;
static int (*real_gethostname)(char *, size_t) = NULL;
static struct passwd *(*real_getpwuid)(uid_t) = NULL;
static int (*real_getpwuid_r)(uid_t, struct passwd *, char *, size_t, struct passwd **) = NULL;
static char *(*real_getlogin)(void) = NULL;

static char spoofed_hostname[256];
static char spoofed_username[64];
static int initialized = 0;

static void generate_spoofed_hostname(void) {
    const char *prefix = "MacBook-Pro-";
    const char chars[] = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    snprintf(spoofed_hostname, sizeof(spoofed_hostname), "%s%c%c%c%c",
             prefix,
             chars[arc4random_uniform((uint32_t)(sizeof(chars) - 1))],
             chars[arc4random_uniform((uint32_t)(sizeof(chars) - 1))],
             chars[arc4random_uniform((uint32_t)(sizeof(chars) - 1))],
             chars[arc4random_uniform((uint32_t)(sizeof(chars) - 1))]);
}

static void generate_spoofed_username(void) {
    const char *names[] = {"alex", "jordan", "casey", "morgan", "riley",
                           "sam", "taylor", "jamie", "quinn", "avery"};
    size_t n = sizeof(names) / sizeof(names[0]);
    const char *name = names[arc4random_uniform((uint32_t)n)];
    strncpy(spoofed_username, name, sizeof(spoofed_username) - 1);
}

__attribute__((constructor))
static void init(void) {
    srandom((unsigned)(time(NULL) ^ getpid()));
    if (!initialized) {
        generate_spoofed_hostname();
        generate_spoofed_username();
        initialized = 1;
    }
}

static void ensure_funcs(void) {
    if (!real___sysctl) real___sysctl = dlsym(RTLD_NEXT, "__sysctl");
    if (!real___sysctlbyname) real___sysctlbyname = dlsym(RTLD_NEXT, "__sysctlbyname");
    if (!real_uname) real_uname = dlsym(RTLD_NEXT, "uname");
    if (!real_gethostname) real_gethostname = dlsym(RTLD_NEXT, "gethostname");
    if (!real_getpwuid) real_getpwuid = dlsym(RTLD_NEXT, "getpwuid");
    if (!real_getpwuid_r) real_getpwuid_r = dlsym(RTLD_NEXT, "getpwuid_r");
    if (!real_getlogin) real_getlogin = dlsym(RTLD_NEXT, "getlogin");
}

/* shared spoofing logic for both sysctl and __sysctl */
static int handle_model_query(int *mib, u_int namelen, void *oldp, size_t *oldlenp) {
    if (namelen != 2 || !oldp || !oldlenp) return -1;
    if (mib[0] == CTL_HW && mib[1] == HW_MODEL) {
        const char *spoofed = "MacBookPro18,3";
        size_t len = strlen(spoofed) + 1;
        if (*oldlenp > len) *oldlenp = len;
        memcpy(oldp, spoofed, (*oldlenp < len) ? *oldlenp : len);
        return 0;
    }
    if (mib[0] == CTL_KERN && mib[1] == KERN_OSRELEASE) {
        const char *spoofed = "24.0.0";
        size_t len = strlen(spoofed) + 1;
        if (*oldlenp > len) *oldlenp = len;
        memcpy(oldp, spoofed, (*oldlenp < len) ? *oldlenp : len);
        return 0;
    }
    return -1;
}

/* Programs link against _sysctl from libSystem — MUST interpose both */
int sysctl(int *mib, u_int namelen, void *oldp, size_t *oldlenp, void *newp, size_t newlen) {
    ensure_funcs();
    int handled = handle_model_query(mib, namelen, oldp, oldlenp);
    if (handled == 0) return 0;
    return real___sysctl ? real___sysctl(mib, namelen, oldp, oldlenp, newp, newlen) : -1;
}

/* Some programs call __sysctl directly from libsystem_kernel */
int __sysctl(int *mib, u_int namelen, void *oldp, size_t *oldlenp, void *newp, size_t newlen) {
    ensure_funcs();
    int handled = handle_model_query(mib, namelen, oldp, oldlenp);
    if (handled == 0) return 0;
    return real___sysctl ? real___sysctl(mib, namelen, oldp, oldlenp, newp, newlen) : -1;
}

/* Programs link against _sysctlbyname from libSystem */
int sysctlbyname(const char *name, void *oldp, size_t *oldlenp, void *newp, size_t newlen) {
    ensure_funcs();
    if (name) {
        if (strcmp(name, "hw.model") == 0 && oldp && oldlenp) {
            const char *spoofed = "MacBookPro18,3";
            size_t len = strlen(spoofed) + 1;
            if (*oldlenp > len) *oldlenp = len;
            memcpy(oldp, spoofed, (*oldlenp < len) ? *oldlenp : len);
            return 0;
        }
        if (strcmp(name, "kern.osrelease") == 0 && oldp && oldlenp) {
            const char *spoofed = "24.0.0";
            size_t len = strlen(spoofed) + 1;
            if (*oldlenp > len) *oldlenp = len;
            memcpy(oldp, spoofed, (*oldlenp < len) ? *oldlenp : len);
            return 0;
        }
    }
    return real___sysctlbyname ? real___sysctlbyname(name, oldp, oldlenp, newp, newlen) : -1;
}

/* Some programs call __sysctlbyname directly */
int __sysctlbyname(const char *name, void *oldp, size_t *oldlenp, void *newp, size_t newlen) {
    ensure_funcs();
    if (name) {
        if (strcmp(name, "hw.model") == 0 && oldp && oldlenp) {
            const char *spoofed = "MacBookPro18,3";
            size_t len = strlen(spoofed) + 1;
            if (*oldlenp > len) *oldlenp = len;
            memcpy(oldp, spoofed, (*oldlenp < len) ? *oldlenp : len);
            return 0;
        }
        if (strcmp(name, "kern.osrelease") == 0 && oldp && oldlenp) {
            const char *spoofed = "24.0.0";
            size_t len = strlen(spoofed) + 1;
            if (*oldlenp > len) *oldlenp = len;
            memcpy(oldp, spoofed, (*oldlenp < len) ? *oldlenp : len);
            return 0;
        }
    }
    return real___sysctlbyname ? real___sysctlbyname(name, oldp, oldlenp, newp, newlen) : -1;
}

int uname(struct utsname *buf) {
    ensure_funcs();
    int ret = real_uname ? real_uname(buf) : -1;
    if (ret == 0 && buf) {
        strncpy(buf->nodename, spoofed_hostname, sizeof(buf->nodename) - 1);
        strncpy(buf->release, "24.0.0", sizeof(buf->release) - 1);
    }
    return ret;
}

int gethostname(char *name, size_t len) {
    ensure_funcs();
    if (name && len > 0) {
        strncpy(name, spoofed_hostname, len - 1);
        name[len - 1] = '\0';
        return 0;
    }
    return real_gethostname ? real_gethostname(name, len) : -1;
}

struct passwd *getpwuid(uid_t uid) {
    ensure_funcs();
    static struct passwd spoofed_pw;
    struct passwd *real = real_getpwuid ? real_getpwuid(uid) : NULL;
    if (real) {
        spoofed_pw = *real;
        spoofed_pw.pw_name = spoofed_username;
        spoofed_pw.pw_gecos = spoofed_username;
        return &spoofed_pw;
    }
    return real;
}

int getpwuid_r(uid_t uid, struct passwd *pwd, char *buffer, size_t bufsize, struct passwd **result) {
    ensure_funcs();
    int ret = real_getpwuid_r ? real_getpwuid_r(uid, pwd, buffer, bufsize, result) : -1;
    if (ret == 0 && pwd && *result) {
        size_t nlen = strlen(spoofed_username) + 1;
        if (buffer && bufsize > nlen) {
            memcpy(buffer, spoofed_username, nlen);
            pwd->pw_name = buffer;
            pwd->pw_gecos = buffer;
        }
        *result = pwd;
    }
    return ret;
}

char *getlogin(void) {
    ensure_funcs();
    return spoofed_username;
}
