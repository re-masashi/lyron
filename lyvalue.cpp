#include <stdio.h>
#include <math.h>
#include <map> 
#include <iostream>
#include <string>
#include <memory>

using namespace std;

extern "C"{
	struct LyClass;
	struct LyValue;

	union LyTypeValue {
		// -1 is null
		int IntVal; // 0
		double DoubleVal; // 1
		bool BoolVal; // 2
		char* StringVal; // 3
		LyValue* FunctionVal; // 4
		LyClass* ClassVal; // 5
	};

	struct LyValue{
		short typeindex;
		union LyTypeValue* val;
	};

	struct LyClass {
		map<string, LyValue* > variables;
		map<string, LyValue* > methods;
		char* name;
	};

	// typedef struct _LyClass LyClass;

	union LyTypeValue* gen_type_val(){
		union LyTypeValue* val;
		val = (union LyTypeValue*) malloc(sizeof(LyTypeValue));
		return val;
	};

	LyValue gen_val(){
		LyValue value;
		value.val = gen_type_val();
		value.typeindex = -1;
		return value;
	};

	LyValue* gen_val_ptr(){
		LyValue* value; 
		value = (LyValue*)malloc(sizeof(LyValue));

		value->val = gen_type_val();
		value->typeindex = -1;
		return value;
	};

	LyClass* create_class(){
		map<string, LyValue* > variables;
		map<string, LyValue* > methods;
		char* name;
		name = (char*)"MyTestFFIClass\0";

		LyValue* value = gen_val_ptr();
		value->typeindex = 0;
		value->val->IntVal = 10;

		variables["testvar"] = value;
		
		for (auto x : variables) 
		   cout << x.first << " | " <<  x.second->typeindex << endl; 

		LyClass* _class;
		_class = (LyClass*)malloc(sizeof(LyClass));
		// printf("testvar: %s\n", variables);
		_class->variables = variables;
		_class->methods = methods;
		_class->name = name;
		return _class;
	};

	void log_value(LyValue value){
		printf("\n");

		switch(value.typeindex){
		case 0:
			printf("val.IntVal []: %d \n", value.val->IntVal);
			printf("val.DoubleVal: %lf \n", value.val->DoubleVal);
			break;
		case 1:
			{
				printf("val.IntVal: %d \n", value.val->IntVal);
				printf("val.DoubleVal []: %lf\n", value.val->DoubleVal);
				break;
			}
		case 2:
			{
				printf("val.BoolVal: %d \n", value.val->BoolVal);
				break;
			}
		case 3:
			printf("val.StringVal: %s\n", value.val->StringVal);
			break;
		case 5:
			printf("val.ClassVal: %s\n", value.val->ClassVal->name);
			printf("]=====START======[\n");
			log_value(*value.val->ClassVal->variables["testvar"]);
			// printf("val.ClassVal vars: %d\n", value.val->ClassVal->variables["testvar"]->val->IntVal);
			// cout << "ok val" << value.val.ClassVal->name << endl;
			printf("]=====END========[\n");
		}
		printf("\n");

	}

	int main(){

		LyValue* value = gen_val_ptr();
		LyClass* _class = create_class();

		printf("Initial typeindex: %d\n", value->typeindex);

		value->typeindex = 0;
		value->val->IntVal = 1;

		log_value(*value);

		value->typeindex = 1;
		value->val->DoubleVal = 1.414141;// gen_double(3, 412);

		log_value(*value);

		value->typeindex = 3;
		value->val->StringVal = (char*)"my name is, my name is\0";// gen_double(3, 412);

		log_value(*value);

		value->typeindex = 5;
		value->val->ClassVal = _class;

		log_value(*value);

		// printf("%d\n", sizeof(val));
		return 0;
	}
}
