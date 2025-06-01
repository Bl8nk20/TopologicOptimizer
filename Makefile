dependency: 
	cd build && cmake .. --graphviz=graph.dot && dot -Tpng graph.dot -o graphImage.png

prepare:
	rm -rf build
	mkdir build
	
compile: 
	cd build && cmake -S .. -B . 

testing:
	make compile && cd build/tests && ./unit_tests