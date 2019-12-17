#include <cstdio>
extern "C" void start_smartpi();
int main(){
    printf("SmartPi Wrapper\n");
    start_smartpi();
    return 0;
}
