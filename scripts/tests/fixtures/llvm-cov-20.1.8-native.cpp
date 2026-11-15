bool choose(bool flag) {
    if (flag) {
        return true;
    }
    return false;
}

int main(int argc, char**) {
    return choose(argc > 1) ? 0 : 1;
}
