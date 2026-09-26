#include <jni.h>

#include <cstdlib>
#include <ctime>
#include <string>

namespace {

std::string g_random;
std::string g_code;
int g_count[10];

bool isAcceptable(const int (&a)[8]) {
    int histogram[10] = {0};
    for (int i = 0; i < 8; ++i) {
        if (++histogram[a[i]] >= 4) {
            return false;
        }
    }
    return true;
}

std::string genRandom() {
    int a[8];
    do {
        srand(clock());
        for (int i = 0; i < 7; ++i) {
            a[i] = rand() % 10;
        }
        a[7] = rand() % 7;
    } while (!isAcceptable(a));

    std::string result;
    for (int i = 0; i < 8; ++i) {
        result += std::to_string(a[i]);
    }
    return result;
}

}  // namespace

extern "C" JNIEXPORT jstring JNICALL
Java_com_xtc_utils_WatchCheckCode_generateCheckCode(JNIEnv* env, jobject, jint type) {
    if (type != 1 && type != 2) {
        return nullptr;
    }

    g_random = genRandom();

    int a[8];
    for (int i = 0; i < 8; ++i) {
        a[i] = g_random[i] - '0';
    }

    const int x = a[7];
    const int value = a[x];

    for (int i = 0; i < 10; ++i) {
        g_count[i] = 0;
    }

    std::string code;
    for (int i = 0; i < 7; ++i) {
        const int offset = (i == x) ? x : value;
        const int digit = (a[i] + offset) % 10;
        ++g_count[digit];
        code += std::to_string(digit);
    }

    const int tail = x ^ type;
    ++g_count[tail];
    code += std::to_string(tail);
    g_code = code;

    return env->NewStringUTF(g_code.c_str());
}

extern "C" JNIEXPORT jboolean JNICALL
Java_com_xtc_utils_WatchCheckCode_check(JNIEnv* env, jobject, jstring input, jint type) {
    if (type != 1 && type != 2) {
        return JNI_FALSE;
    }
    if (input == nullptr) {
        return JNI_FALSE;
    }

    const char* chars = env->GetStringUTFChars(input, nullptr);
    std::string str(chars);
    env->ReleaseStringUTFChars(input, chars);

    if (str.size() != 8) {
        return JNI_FALSE;
    }

    int d[8];
    for (int i = 0; i < 8; ++i) {
        d[i] = str[i] - '0';
    }

    int histogram[10] = {0};
    for (int i = 0; i < 8; ++i) {
        ++histogram[d[i]];
    }

    bool sameHistogram = true;
    for (int k = 0; k < 10; ++k) {
        if (histogram[k] != g_count[k]) {
            sameHistogram = false;
            break;
        }
    }
    if (sameHistogram) {
        return JNI_FALSE;
    }

    const std::string target = g_random.substr(0, 7);

    for (int i = 0; i < 8; ++i) {
        const int k = d[i] ^ type;
        if (k >= 7) {
            continue;
        }

        int smaller[7];
        int n = 0;
        for (int j = 0; j < 8; ++j) {
            if (j != i) {
                smaller[n++] = d[j];
            }
        }

        const int pivot = (smaller[k] - k + 10) % 10;

        std::string built;
        for (int j = 0; j < 7; ++j) {
            const int digit = (j == k) ? pivot : (smaller[j] - pivot + 10) % 10;
            built += std::to_string(digit);
        }

        if (built == target) {
            return JNI_TRUE;
        }
    }

    return JNI_FALSE;
}
