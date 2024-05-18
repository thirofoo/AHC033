.PHONY: test all

# Check if the argument is a valid number between 0 and 99
VALID_ARG = $(shell echo $(CASE) | grep -E '^[0-9]+$$')

# Format the CASE variable with leading zeros to match the required filename format
FORMAT_CASE = $(shell printf "%04d" $(CASE))

# ANSI color codes
COLOR_RED = "\033[31m"
COLOR_BLUE = "\033[34m"
COLOR_RESET = "\033[37m"

test:
	@if [ -z "$(CASE)" ]; then \
		echo "Error: CASE argument is required."; \
		exit 1; \
	elif [ -z "$(VALID_ARG)" ] || [ $(CASE) -lt 0 ] || [ $(CASE) -gt 99 ]; then \
		echo "Error: Invalid CASE argument. Must be a number between 0 and 99."; \
		exit 1; \
	fi
	@cargo run < ./tools/in/$(FORMAT_CASE).txt > ./tools/out/$(FORMAT_CASE).txt 2> /dev/null
	@cd tools && cargo run -r --bin vis ./in/$(FORMAT_CASE).txt ./out/$(FORMAT_CASE).txt

all:
	@# ./tools/scores_now clear
	@echo -n > ./tools/scores_now

	@# Run all test cases and save the scores
	@TEST_CASES=100; \
	PERCENT_CHANGE_SUM=0; \
	for i in $$(seq 0 $$((TEST_CASES-1))); do \
		make -s test CASE=$$i 2>> /dev/null | cut -c 9- >> ./tools/scores_now; \
		RESULT=$$(tail -n 1 ./tools/scores_now); \
		BEST_SCORE=$$(sed -n "$$(($$i+1))p" ./tools/scores_best); \
		if [ -n "$$BEST_SCORE" ]; then \
			PERCENT_CHANGE=$$(awk "BEGIN {print ($$RESULT / $$BEST_SCORE) * 100}"); \
			if [ $$(awk "BEGIN {print ($$PERCENT_CHANGE < 100)}") -eq 1 ]; then \
				COLOR=$$(echo $(COLOR_BLUE)); \
			elif [ $$(awk "BEGIN {print ($$PERCENT_CHANGE > 100)}") -eq 1 ]; then \
				COLOR=$$(echo $(COLOR_RED)); \
			else \
				COLOR=$$(echo $(COLOR_RESET)); \
			fi; \
			echo "Test case $$i : $$RESULT ( $${COLOR}$$PERCENT_CHANGE% $$(echo $(COLOR_RESET)))"; \
			PERCENT_CHANGE_SUM=$$(awk "BEGIN {print ($$PERCENT_CHANGE_SUM + $$PERCENT_CHANGE)}"); \
		else \
			echo "Test case $$i : $$RESULT (No best score)"; \
		fi; \
	done; \
	PERCENT_CHANGE_SUM=$$(awk "BEGIN {print ($$PERCENT_CHANGE_SUM / $$TEST_CASES)}"); \
	echo "Average percent change: $$PERCENT_CHANGE_SUM%"
