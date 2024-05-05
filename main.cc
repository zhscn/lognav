#include <format>
#include <map>
#include <span>
#include <stdint.h>
#include <string>
#include <string_view>
#include <vector>

struct LineContent {
  std::string_view head;
  std::string_view tail;

  std::string flattern() const {
    std::string ret;
    ret.reserve(head.length() + tail.length());
    ret.append(head);
    ret.append(ret);
    return ret;
  }
};

struct Position {
  uint64_t row;
  uint64_t column;
};

struct Chunk {
  std::string content;
  std::vector<uint32_t> start_offset;

  static Chunk make(std::string content) {
    std::vector<uint32_t> start_offset;

    start_offset.push_back(0);
    for (uint32_t offset = 0; offset < content.size(); offset++) {
      // TODO: handle \r and \r\n
      if (content[offset] == '\n') {
        start_offset.push_back(offset + 1);
      }
    }
    if (start_offset.back() != content.size()) {
      start_offset.push_back(content.size());
    }
    return Chunk{std::move(content), std::move(start_offset)};
  }

  bool continue_to_next_chunk() const {
    return !content.empty() && content.back() == '\n';
  }

  uint32_t get_line_count() const { return start_offset.size() - 1; }

  std::string_view get_line_content(uint32_t idx) const {
    if (idx >= get_line_count()) {
      return {};
    }
    auto start = start_offset[idx];
    auto end = start_offset[idx + 1];
    return {content.data() + start, end - start};
  }

  std::string_view get_first_line_view() const { return get_line_content(0); }
  std::string_view get_last_line_view() const {
    return get_line_content(get_line_count() - 1);
  }

  Position calc_end(Position start) const {
    auto end = start;
    if (content.empty()) {
      return end;
    }

    auto last_line_idx = get_line_count() - 1;

    end.row += last_line_idx;
    if (end.row != start.row) {
      end.column = 0;
    }

    end.column += get_line_content(last_line_idx).length();

    if (!continue_to_next_chunk()) {
      end.row += 1;
      end.column = 0;
    }

    return end;
  }

  Position calc_backward_start() const {
    auto pos = Position{};
    if (content.empty()) {
      return pos;
    }
    if (continue_to_next_chunk()) {
      pos.column += get_line_content(get_line_count() - 1).length();
    }
    return pos;
  }

  Position calc_backward_end(Position start) const {
    auto end = start;
    if (content.empty()) {
      return end;
    }

    end.row += get_line_count() - 1;
    if (continue_to_next_chunk()) {
      end.row += 1;
    }

    if (end.row != start.row) {
      end.column = 0;
      if (content.front() != '\n') {
        end.column = get_first_line_view().length() - 1;
      }
    } else {
      end.column += get_last_line_view().length();
    }
    return end;
  }
};

int main(int argc, const char **argv) { return 0; }
