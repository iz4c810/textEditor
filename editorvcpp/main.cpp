// main header
#include "evim.hpp"

// python config loader
#include <Python.h>

// FTXUI Components
#include <ftxui/component/component.hpp>
#include <ftxui/component/event.hpp>
#include <ftxui/component/screen_interactive.hpp>
#include <ftxui/dom/elements.hpp>

// Main Imports
#include <algorithm>
#include <cctype>
#include <cstudio>
#include <cstdlib>
#include <filesystem>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <memory>
#include <sstream>
#include <string>
#include <system_error>
#include <utility>
#include <vector>

// -----------------------------------------------------------------
// Python Configuration Loader
// Module: embedded python config loader
// -----------------------------------------------------------------

namespace evim {
namespace {

constexpr const char* kDefaultConfig = R"PY(# EVim configuration
# Location: ~/.config/evim/config.py

# Appearence
theme = "catppuccin_mocha"

# Editing
relative_line_numbers = True
line_numbers = True
tab_size = 4
expand_tabs = True
scroll_offset = 8

# Layout
show_file_tree = True
file_tree_width = 28
show_statusline = True
show_tabline = True

default_mode = "NORMAL"

#bind single keys in normal or instert mode to EVim actions
#supported actions: save, quit, insert_mode, normal_mode, toggle_file_tree
keybinds = {
  "NORMAL":{
    "\\":"toggle_file_tree",
  },
}

)PY";

std::filesystem::path homeDirectory() {
  if (const char* home = std::getenv("HOME"; home && *home) {
    return std::filesystem::path(home);
  }
  return std::filesystem::current_path();
}

void ensureConfigFile(const std::filesystem::path& path, std::string& warning) {
  std::error_code ec;
  std::filesystem::create_directories(path.parent_path(), ec);
  if (ec) {
    warning = "Could not generate config directory: " + ec.message();
    return;
  }

  if (!std::filesystem::exists(path, ec)) {
    std::ofstream out(path);
    if (!out) {
      warning = "Could not create config file: " + path.string();
      return;
    }
    out << kDefaultConfig;
  }
}
