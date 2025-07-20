install:
	sudo apt install gcc g++ cmake make doxygen git llvm pkg-config curl zip unzip tar python3-dev clang-format clang-tidy

prepare:
	rm -rf build
	mkdir build
compile: 
	cd build && cmake -S .. -B . 
	
dependency: 
	cd build && cmake .. --graphviz=graph.dot && dot -Tpng graph.dot -o graphImage.png