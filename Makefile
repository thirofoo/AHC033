test:
	@clear
	@echo "単体 test"
	g++ main.cpp -o main -std=c++17
	@./main < _in > _out 2> _err
	@cd tools; \
    cargo run -r --bin vis ../_in ../_out

all:
	@clear
	@echo "全体 test"
	@cd tools; \
	./test.sh

sample:
	@clear
	@echo "Sample test"
	@python3 sample.py < _in > _out
	@cd tools; \
    cargo run -r --bin vis ../_in ../_out
