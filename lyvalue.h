#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>

struct LyClass;
struct LyValue;
struct Map;
struct LyArray;

union LyTypeValue {
	// -1 is null
	int IntVal; // 0
	double DoubleVal; // 1
	bool BoolVal; // 2
	char* StringVal; // 3
	LyValue* FunctionVal; // 4
	LyClass* ClassVal; // 5
	Map* DictVal; // 6
	LyArray* ArrayVal;  // 7
};

struct LyValue{
	short typeindex;
	union LyTypeValue* val;
};

struct LyArray{
	int size;
	int max_size;
	LyValue** values;
};

struct Map{
	int size;
	int max_size;
	LyValue** values;
	char** keys;
};

struct LyClass {
	Map* variables;
	Map* methods;
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

	Map* create_map(){
		Map* map;
		map = (Map*) malloc(sizeof(Map));
		map->size = 0;
		map->max_size = 256;
		map->values = (LyValue**) malloc(map->max_size* sizeof(LyValue*));
		map->keys = (char**) malloc(map->max_size * sizeof(char*));
		return map;
	}

	LyArray* create_lyarray(){
		LyArray* arr;
		arr = (LyArray*) malloc(sizeof(LyArray));
		arr->size = 0;
		arr->max_size = 256;
		arr->values = (LyValue**) malloc(arr->max_size* sizeof(LyValue*));
		return arr;
	}

	int map_get_index(Map* map, char* key){
		for (int i = 0; i < map->size; i++)
		{
			if (map->keys[i]==key)
			{
				// printf("key exists\n");
				return i;
			}
		}
		return -1;
	}

	void map_insert(Map* map, char* key, LyValue* value){
		int prev_occurance = map_get_index(map, key);

		if (map->max_size==map->size && prev_occurance!=-1) // max filled
		{
			map->values = (LyValue**) realloc(map, map->size+256* sizeof(LyValue*));
			map->keys = (char**) realloc(map, map->size+256* sizeof(char*));
			map->max_size = map->max_size+256;
		}
		if (prev_occurance!=-1)
		{
			map->values[prev_occurance];
		}else{
			// length of array = map->size-1
			map->keys[map->size] = key;
			map->values[map->size] = value;
		}
		map->size+=1;
	}

	LyClass* create_class(){
		Map variables;
		Map methods;
		char* name;
		name = (char*)"MyTestFFIClass\0";

		LyValue* value = gen_val_ptr();
		value->typeindex = 0;
		value->val->IntVal = 10;

		// variables["testvar"] = value;
		
		// for (auto x : variables) 
		//    cout << x.first << " | " <<  x.second->typeindex << endl; 

		LyClass* _class;
		_class = (LyClass*)malloc(sizeof(LyClass));
		// printf("testvar: %s\n", variables);
		_class->variables = create_map();
		_class->methods = create_map();
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
			printf("val.ClassVal name: %s\n", value.val->ClassVal->name);
			printf("]=====START======[\n");
			// log_value(*value.val->ClassVal->variables["testvar"]);
			// cout << "ok val" << value.val.ClassVal->name << endl;
			for (int i = 0; i < value.val->ClassVal->variables->size; i++)
			{
				printf("key: %s \n", value.val->ClassVal->variables->keys[i]);
				// log_value(*value.val->ClassVal->variables.values[i]);
			}
			printf("]=====END========[\n");
			break;
		case 6:
			printf("val.DictVal length=%d \n", value.val->ArrayVal->size);
			printf("]=====START DictVal======[\n");
			for (int i = 0; i < value.val->DictVal->size; i++)
			{
				printf("key: %s \n", value.val->DictVal->keys[i]);
				log_value(*value.val->DictVal->values[i]);
			}
			printf("]=====END DictVal========[\n");
			break;
		case 7:
			printf("val.ArrayVal length=%d \n", value.val->ArrayVal->size);
			printf("]=====START ArrayVal======[\n");
			for (int i = 0; i < value.val->ArrayVal->size; i++)
			{
				log_value(*value.val->ArrayVal->values[i]);
			}
			printf("]=====END ArrayVal========[\n");
			break;
		default:
			printf("Null typeindex=%d\n", value.typeindex);
		}
		// printf("\n");
	}

	// int main(){

	// 	LyValue* value = gen_val_ptr();
	// 	LyClass* _class = create_class();

	// 	Map* map = create_map();
	// 	map_insert(map, (char*)"testvar\0", gen_val_ptr());

	// 	_class->variables = map;

	// 	LyArray* arr = create_lyarray();
	// 	// map_insert(map, (char*)"testvar\0", gen_val_ptr());

	// 	printf("Initial typeindex: %d\n", value->typeindex);

	// 	value->typeindex = 0;
	// 	value->val->IntVal = 1;

	// 	log_value(*value);

	// 	value->typeindex = 1;
	// 	value->val->DoubleVal = 1.414141;// gen_double(3, 412);

	// 	log_value(*value);

	// 	value->typeindex = 3;
	// 	value->val->StringVal = (char*)"my name is, my name is\0";// gen_double(3, 412);

	// 	log_value(*value);

	// 	value->typeindex = 5;
	// 	value->val->ClassVal = _class;

	// 	log_value(*value);

	// 	value->typeindex = 6;
	// 	value->val->DictVal = map;

	// 	log_value(*value);

	// 	value->typeindex = 7;
	// 	value->val->ArrayVal = arr;

	// 	log_value(*value);

	// 	// printf("%d\n", sizeof(val));
	// 	return 0;
	// }