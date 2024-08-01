#include <lyvalue.cpp>
#include <stdio.h>

extern "C"{
	LyValue* call_printf(int arity, LyValue** args){
		printf("ok before access\n");
		printf("typeindex %d\n", args[0]->typeindex);
		log_value(*args[0]);
		return gen_val_ptr();
	}
}