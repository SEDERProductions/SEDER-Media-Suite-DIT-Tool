#pragma once

#include "DitResult.h"

#include <functional>

class SederFfi {
public:
    static DitResult compare(const DitRequestData &request, const std::function<void(const DitProgress &)> &progress);
};
